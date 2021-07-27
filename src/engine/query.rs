use std::{collections::HashMap, sync::RwLock};

use super::environment::Database;

use crate::structs::{
    fact::{ValueLine, ValueTable},
    rule::{Expr, FactQuery, Pattern, RuleBody, RuleTable},
    scope::{Scope, SimpleScope},
    value::{Handle, Value},
};

// use rayon::prelude::*;
use sexpr_ir::gast::symbol::Symbol;

fn unify(pattern: &Expr, value: &Value, env: &Handle<Scope>) -> Result<(), ()> {
    match pattern {
        Expr::Value(v) => {
            if v == value {
                Ok(())
            } else {
                Err(())
            }
        }
        Expr::Variable(k) => {
            if let Some(ref v) = env.find(k) {
                if v == value {
                    Ok(())
                } else {
                    Err(())
                }
            } else {
                env.set(k, value);
                Ok(())
            }
        }
        Expr::FunctionCall(_) => Err(()),
    }
}

fn matching(
    pattern: &Pattern,
    value: &Expr,
    record: &mut HashMap<Handle<Symbol>, Expr>,
) -> Result<(), ()> {
    match pattern {
        Pattern::Ignore => Ok(()),
        Pattern::Variable(k) => {
            if let Some(c) = record.get(k) {
                if c != value {
                    return Err(());
                }
            } else {
                record.insert(k.clone(), value.clone());
            }
            Ok(())
        }
        Pattern::Constant(c) => {
            if let Expr::Value(value) = value {
                if c == value {
                    Ok(())
                } else {
                    Err(())
                }
            } else {
                Err(())
            }
        }
        Pattern::Tuple(_) => todo!(),
        Pattern::List(_, _) => todo!(),
    }
}

fn swap_result<T, E>(i: Result<T, E>) -> Result<E, T> {
    match i {
        Ok(x) => Err(x),
        Err(x) => Ok(x),
    }
}

fn query_value_line(this: &ValueLine, env: &Handle<Scope>, prarms: &[Expr]) -> Result<Handle<Scope>, ()> {
    // if err return err
    let env = env.new_level(SimpleScope::new());
    for (value, pattern) in this.0.iter().zip(prarms.iter()) {
        if unify(pattern, value, &env).is_err() {
            return Err(());
        }
    }
    Ok(env)
}

fn query_rule_body(
    this: &RuleBody,
    env: &Handle<Database>,
    scope: &Handle<Scope>,
    prarms: &[Expr],
) -> Result<Handle<Scope>, ()> {
    let new_scope = scope.new_level(SimpleScope::new());

    let mut capture = HashMap::new();
    this.prarms
        .iter()
        .zip(prarms.iter())
        .try_for_each(|(pattern, value)| matching(pattern, value, &mut capture))?;

    for query in this.bodys.iter() {
        if query_fact(query, env, &new_scope, &mut capture).is_none() {
            return Err(());
        }
    }
    
    Ok(new_scope)
}

pub fn query_value_table(
    this: &ValueTable,
    env: &Handle<Scope>,
    prarms: &[Expr],
) -> Option<Handle<Scope>> {
    let r: Result<Vec<_>, ()> = this
        .0
        .par_iter()
        .map(|values| swap_result(query_value_line(values, &env.new_level(SimpleScope::new()), prarms)))
        .collect();
    // if err return ok
    if r.is_err() {
        Some(new_env)
    } else {
        // traceback
        None
    }
}

pub fn query_rule_table(
    this: &RuleTable,
    env: &Handle<Database>,
    scope: &Handle<Scope>,
    prarms: &[Expr],
) -> Option<Handle<Scope>> {
    let r: Result<_, _> = this
        .0
        .par_iter()
        .try_for_each(|value| swap_result(query_rule_body(value, env, &scope.new_level(SimpleScope::new()), &prarms)));
    // if err return ok
    if let Err(r) = r {
        Some(r)
    } else {
        // traceback
        None
    }
}

pub fn query_fact(
    this: &FactQuery,
    env: &Handle<Database>,
    scope: &Handle<Scope>,
    prarms: &mut HashMap<Handle<Symbol>, Expr>,
) -> Option<()> {
    let prarms: Option<Vec<_>> = this.args.par_iter().map(|x| binding(x, prarms)).collect();
    let prarms = prarms?;
    let k = (this.name.clone(), prarms.len());
    let record = env.rules.read().unwrap();
    if let Some(rules) = record.0.get(&k) {
        return if let Some(x) = query_rule_table(rules, env, scope, &prarms) {
            x.flatten().0
                .read()
                .unwrap()
                .par_iter()
                .try_for_each(|(k, v)| {
                    if let Some(ref x) = scope.find(k) {
                        if x == v {
                            Some(())
                        } else {
                            None
                        }
                    } else {
                        scope.set(k, v);
                        Some(())
                    }
                })
        } else {
            None
        };
    }
    let record = env.facts.read().unwrap();
    if let Some(facts) = record.0.get(&k) {
        return if let Some(x) = query_value_table(facts, scope, &prarms) {
            x.flatten()
                .0
                .read()
                .unwrap()
                .par_iter()
                .try_for_each(|(k, v)| {
                    if let Some(ref x) = scope.find(k) {
                        if x == v {
                            Some(())
                        } else {
                            None
                        }
                    } else {
                        scope.set(k, v);
                        Some(())
                    }
                })
        } else {
            None
        };
    }
    None
}

fn binding(x: &Expr, prarms: &RwLock<HashMap<Handle<Symbol>, Expr>>) -> Option<Expr> {
    match x {
        Expr::Variable(k) => {
            if let Some(x) = prarms.read().unwrap().get(k) {
                Some(x.clone())
            } else {
                prarms
                    .write()
                    .unwrap()
                    .get(k)
                    .cloned()
                    // .map_or_else(|| None, |x| Some(x.clone()))
            }
        }
        _ => Some(x.clone()),
    }
}
