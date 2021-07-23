use std::{collections::HashMap, sync::RwLock};

use super::environment::Database;

use crate::structs::{
    fact::{ValueLine, ValueTable},
    rule::{Expr, FactQuery, Pattern, RuleBody, RuleTable},
    scope::{Scope, SimpleScope},
    value::{Handle, Value},
};

use rayon::prelude::*;
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
    record: &RwLock<HashMap<Handle<Symbol>, Expr>>,
) -> Result<(), ()> {
    match pattern {
        Pattern::Ignore => Ok(()),
        Pattern::Variable(k) => {
            if let Some(c) = record.read().unwrap().get(k) {
                if c != value {
                    return Err(());
                }
            } else {
                record.write().unwrap().insert(k.clone(), value.clone());
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

fn query_value_line(this: &ValueLine, env: &Handle<Scope>, prarms: &[Expr]) -> Result<(), ()> {
    // if err return err
    this.0
        .par_iter()
        .zip(prarms.par_iter())
        .try_for_each(|(value, pattern)| unify(pattern, value, env))?;
    Ok(())
}

fn query_rule_body(
    this: &RuleBody,
    env: &Handle<Database>,
    scope: &Handle<Scope>,
    prarms: &[Expr],
) -> Result<Handle<Scope>, ()> {
    let new_scope = scope.new_level(SimpleScope::new());

    let capture = RwLock::new(HashMap::new());
    this.prarms
        .par_iter()
        .zip(prarms.par_iter())
        .try_for_each(|(pattern, value)| matching(pattern, value, &capture))?;

    this.bodys
        .par_iter()
        .try_for_each(|x| query_fact(x, env, &new_scope, &capture))?;
    Ok(new_scope)
}

fn query_value_table(
    this: &ValueTable,
    env: &Handle<Scope>,
    prarms: &[Expr],
) -> Result<Handle<Scope>, ()> {
    let new_env = env.new_level(SimpleScope::new());
    let r: Result<Vec<_>, ()> = this
        .0
        .par_iter()
        .map(|values| swap_result(query_value_line(values, &new_env, prarms)))
        .collect();
    // if err return ok
    if r.is_err() {
        Ok(new_env)
    } else {
        // traceback
        Err(())
    }
}

fn query_rule_table(
    this: &RuleTable,
    env: &Handle<Database>,
    scope: &Handle<Scope>,
    prarms: &[Expr],
) -> Result<Handle<Scope>, ()> {
    let new_scope = scope.new_level(SimpleScope::new());
    let r: Result<_, _> = this
        .0
        .par_iter()
        .try_for_each(|value| swap_result(query_rule_body(value, env, &new_scope, &prarms)));
    // if err return ok
    if let Err(r) = r {
        Ok(r)
    } else {
        // traceback
        Err(())
    }
}

fn query_fact(
    this: &FactQuery,
    env: &Handle<Database>,
    scope: &Handle<Scope>,
    prarms: &RwLock<HashMap<Handle<Symbol>, Expr>>,
) -> Result<(), ()> {
    let prarms: Result<Vec<_>, ()> = this.args.par_iter().map(|x| binding(x, prarms)).collect();
    let prarms = prarms?;

    let k = (this.name.clone(), prarms.len());
    let record = env.rules.read().unwrap();
    if let Some(rules) = record.0.get(&k) {
        return if let Ok(x) = query_rule_table(rules, env, scope, &prarms) {
            x.flatten()
                .0
                .read()
                .unwrap()
                .par_iter()
                .try_for_each(|(k, v)| {
                    if let Some(ref x) = scope.find(k) {
                        if x == v {
                            Ok(())
                        } else {
                            Err(())
                        }
                    } else {
                        scope.set(k, v);
                        Ok(())
                    }
                })
        } else {
            Err(())
        };
    }
    let record = env.facts.read().unwrap();
    if let Some(facts) = record.0.get(&k) {
        return if let Ok(x) = query_value_table(facts, scope, &prarms) {
            x.flatten()
                .0
                .read()
                .unwrap()
                .par_iter()
                .try_for_each(|(k, v)| {
                    if let Some(ref x) = scope.find(k) {
                        if x == v {
                            Ok(())
                        } else {
                            Err(())
                        }
                    } else {
                        scope.set(k, v);
                        Ok(())
                    }
                })
        } else {
            Err(())
        };
    }
    Err(())
}

fn binding(x: &Expr, prarms: &RwLock<HashMap<Handle<Symbol>, Expr>>) -> Result<Expr, ()> {
    match x {
        Expr::Variable(k) => {
            if let Some(x) = prarms.read().unwrap().get(k) {
                Ok(x.clone())
            } else {
                prarms
                    .write()
                    .unwrap()
                    .get(k)
                    .map_or_else(|| Err(()), |x| Ok(x.clone()))
            }
        }
        _ => Ok(x.clone()),
    }
}
