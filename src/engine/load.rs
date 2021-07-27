use std::{collections::HashMap, sync::RwLock};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sexpr_ir::gast::{symbol::Symbol, GAst, Handle};
use sexpr_process::capture::{Capture, Catch};

use crate::{
    engine::query::query_fact,
    structs::{
        fact::{FactRecord, ValueLine, ValueTable},
        rule::{Expr, FactQuery, Pattern, RuleBody, RuleRecord, RuleTable},
        scope::{Scope, SimpleScope},
    },
};

use super::{environment::Database, parser::FromGast, utils::*};

use super::eval::eval_value;

pub trait Loader {
    fn load(&mut self, env: &Handle<Scope>, input: &GAst) -> Option<()>;
}

impl Loader for FactRecord {
    fn load(&mut self, env: &Handle<Scope>, input: &GAst) -> Option<()> {
        let r = FACT_PATTERN.catch(input).ok()?;
        let r: HashMap<Handle<Symbol>, Capture> = r.into_iter().collect();

        let name = r.get(&Symbol::new("name")).unwrap().get_one().unwrap();
        let name = name.get_const()?.get_sym()?;

        let exprs = r.get(&Symbol::new("exprs")).unwrap().get_many().unwrap();
        let exprs: Option<Handle<[_]>> = exprs
            .iter()
            .map(|x| Expr::from_gast(x).map(|x| eval_value(&x, env)).flatten())
            .collect();
        let exprs = exprs?;

        let key = (name, exprs.len());
        if let Some(x) = self.0.get_mut(&key) {
            x.0.push(ValueLine(exprs));
        } else {
            self.0.insert(key, ValueTable(vec![ValueLine(exprs)]));
        }
        Some(())
    }
}

impl Loader for RuleRecord {
    fn load(&mut self, _: &Handle<Scope>, input: &GAst) -> Option<()> {
        let r = RULE_PATTERN.catch(input).ok()?;
        let r: HashMap<Handle<Symbol>, Capture> = r.into_iter().collect();

        let prarms = r.get(&Symbol::new("prarms")).unwrap().get_one().unwrap();
        let prarms = RULE_PARAMS_PATTERN.catch(prarms).ok()?;
        let prarms: HashMap<Handle<Symbol>, Capture> = prarms.into_iter().collect();

        let name = prarms.get(&Symbol::new("name")).unwrap().get_one().unwrap();
        let name = name.get_const()?.get_sym()?;

        let args = prarms
            .get(&Symbol::new("args"))
            .unwrap()
            .get_many()
            .unwrap();
        let args: Option<Handle<[_]>> = args.iter().map(Pattern::from_gast).collect();
        let args = args?;

        let exprs = r.get(&Symbol::new("exprs")).unwrap().get_many().unwrap();
        let exprs: Option<Handle<[_]>> = exprs.iter().map(FactQuery::from_gast).collect();
        let exprs = exprs?;

        let key = (name, prarms.len());
        let value = RuleBody {
            prarms: args,
            bodys: exprs,
        };

        if let Some(x) = self.0.get_mut(&key) {
            x.0.push(value);
        } else {
            self.0.insert(key, RuleTable(vec![value]));
        }
        Some(())
    }
}


    fn database_load(this: &Handle<Database>, env: &Handle<Scope>, input: &GAst) -> Option<()> {
        let r = this.facts.write().unwrap().load(env, input);
        if r.is_some() {
            return Some(());
        }
        let r = this.rules.write().unwrap().load(env, input);
        if r.is_some() {
            return Some(());
        }
        if let Ok(capture) = DEFINE_PATTERN.catch(input) {
            let capture: HashMap<Handle<Symbol>, Capture> = capture.into_iter().collect();
            let name = capture
                .get(&Symbol::new("name"))
                .unwrap()
                .get_one()
                .unwrap()
                .get_const()?
                .get_sym()?;
            let expr = capture
                .get(&Symbol::new("expr"))
                .unwrap()
                .get_one()
                .unwrap();
            let expr = Expr::from_gast(expr)?;
            let value = eval_value(&expr, env)?;
            env.set(&name, &value);
            Some(())
        } else {
            None
        }
    }

pub fn apply_query(env: &Handle<Database>, scope: &Handle<Scope>, input: &GAst) -> Option<SimpleScope> {
    // parse
    let capture = QUERY_PATTERN.catch(input).ok()?;
    let capture: HashMap<Handle<Symbol>, Capture> = capture.into_iter().collect();
    let prarms = capture
        .get(&Symbol::new("prarms"))
        .unwrap()
        .get_one()
        .unwrap();
    let prarms = QUERY_PARAMS_PATTERN.catch(prarms).ok()?;
    let (cap_name, args) = prarms.first()?;
    debug_assert_eq!(cap_name.0.as_str(), "args");
    let args: Option<Box<[_]>> = args
        .get_many()?
        .iter()
        .map(|x| x.get_const()?.get_sym())
        .collect();
    let args = args?;
    let exprs: Option<Box<[_]>> = capture
        .get(&Symbol::new("exprs"))
        .unwrap()
        .get_many()
        .unwrap()
        .iter()
        .map(FactQuery::from_gast)
        .collect();
    let exprs = exprs?;

    // init env: new scope
    let capture: HashMap<_, _> = args
        .iter()
        .map(|x| (x.clone(), Expr::Variable(x.clone())))
        .collect();
    let capture = RwLock::new(capture);
    let new_scope = scope.new_level(SimpleScope::new());
    // eval
    exprs
        .par_iter()
        .try_for_each(|x| query_fact(x, env, &new_scope, &capture))?;
    Some(new_scope.this_level.clone())
}


pub fn repl_eval(db: &Handle<Database>, env: &Handle<Scope>, input: &GAst) -> Option<Option<SimpleScope>> {
    if database_load(db, env, input).is_some() {
        return Some(None);
    }
    apply_query(db, env, input).map(Some)
}