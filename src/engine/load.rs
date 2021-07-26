use std::collections::HashMap;


use sexpr_ir::{
    gast::{symbol::Symbol, GAst, Handle},
};
use sexpr_process::capture::{Capture, Catch};

use crate::structs::{fact::{FactRecord, ValueLine, ValueTable}, rule::{Expr, FactQuery, Pattern, RuleBody, RuleRecord, RuleTable}, scope::Scope};

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

        let name = r.get(&Symbol::new("name")).unwrap().get_one().unwrap();
        let name = name.get_const()?.get_sym()?;

        let args = r.get(&Symbol::new("args")).unwrap().get_many().unwrap();
        let args: Option<Handle<[_]>> =
            args.iter().map(Pattern::from_gast).collect();
        let args = args?;

        let exprs = r.get(&Symbol::new("exprs")).unwrap().get_many().unwrap();
        let exprs: Option<Handle<[_]>> = exprs
            .iter()
            .map(FactQuery::from_gast)
            .collect();
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


impl Loader for Database {
    fn load(&mut self, env: &Handle<Scope>, input: &GAst) -> Option<()> {
        let r = self.facts.write().unwrap().load(env, input);
        if r.is_some() {
            return Some(());
        }
        let r = self.rules.write().unwrap().load(env, input);
        if r.is_some() {
            return Some(());
        }
        if let Ok(capture) = DEFINE_PATTERN.catch(input) {
            let capture: HashMap<Handle<Symbol>, Capture> = capture.into_iter().collect();
            let name = capture
                .get(&Symbol::new("name")).unwrap()
                .get_one().unwrap()
                .get_const()?
                .get_sym()?;
            let expr = capture
                .get(&Symbol::new("expr")).unwrap()
                .get_one().unwrap();
            let expr = Expr::from_gast(expr)?;
            let value = eval_value(&expr, env)?;
            env.set(&name, &value);
            Some(())
        } else {
            None
        }
    }
}