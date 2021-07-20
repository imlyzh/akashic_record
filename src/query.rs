use std::collections::HashMap;

use crate::{
    environment::{Database, Env},
    fact::{ValueLine, ValueTable},
    rule::{Expr, Fact, Pattern, RuleBody, RuleTable},
    scope::{Scope, SimpleScope},
    value::{Handle, Value},
};

use rayon::prelude::*;
use sexpr_ir::gast::symbol::Symbol;

fn query_value_line(this: &ValueLine, env: &Handle<Scope>, prarms: &[Expr]) -> Result<(), ()> {
    // if err return err
    this.0
        .par_iter()
        .zip(prarms.par_iter())
        .try_for_each(|(value, pattern)| unify(pattern, value, env))?;
    Ok(())
}

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

fn swap_result<T, E>(i: Result<T, E>) -> Result<E, T> {
    match i {
        Ok(x) => Err(x),
        Err(x) => Ok(x),
    }
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

fn matching(
    pattern: &Pattern,
    value: &Expr,
    scope: &Handle<Scope>,
) -> Result<Vec<(Handle<Symbol>, Expr)>, ()> {
    match pattern {
        Pattern::Ignore => Ok(vec![]),
        Pattern::Variable(k) => Ok(vec![(k.clone(), value.clone())]),
        Pattern::Constant(c) => {
            if let Expr::Value(value) = value {
                if c == value {
                    Ok(vec![])
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

fn query_rule_body(
    this: &RuleBody,
    env: &Handle<Database>,
    scope: &Handle<Scope>,
    prarms: &[Expr],
) -> Result<Handle<Scope>, ()> {
    let new_scope = scope.new_level(SimpleScope::new());

    let r: Result<Vec<_>, _> = this
        .prarms
        .par_iter()
        .zip(prarms.par_iter())
        .map(|(pattern, value)| matching(pattern, value, &new_scope))
        .collect();

    let r = r?;

    let capture: HashMap<Handle<Symbol>, Expr> = r.into_par_iter().flatten().collect();

    this.bodys
        .par_iter()
        .try_for_each(|x| query_fact(x, env, &new_scope, &capture))?;
    Ok(new_scope)
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
    this: &Fact,
    env: &Handle<Database>,
    scope: &Handle<Scope>,
    prarms: &HashMap<Handle<Symbol>, Expr>,
) -> Result<(), ()> {
    let prarms: Result<Vec<_>, ()> = this
        .args
        .iter()
        .map(|x| binding(x, prarms, scope))
        .collect();
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

fn binding(
    x: &Expr,
    prarms: &HashMap<Handle<Symbol>, Expr>,
    scope: &Handle<Scope>,
) -> Result<Expr, ()> {
    match x {
        Expr::Variable(k) => {
            if let Some(x) = prarms.get(k) {
                Ok(x.clone())
            } else {
                prarms.get(k).map_or_else(|| Err(()), |x| Ok(x.clone()))
            }
        }
        _ => Ok(x.clone()),
    }
}
