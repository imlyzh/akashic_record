use std::collections::HashMap;

use lazy_static::*;
use sexpr_ir::{
    gast::{symbol::Symbol, GAst, Handle},
    syntax::sexpr::one_unit_parse,
};
use sexpr_process::{
    capture::{Capture, Catch},
    pattern::Pattern,
};

use crate::structs::{
    fact::{FactRecord, ValueLine, ValueTable},
    rule::Expr,
    scope::Scope,
    value::Value,
};

use rayon::prelude::*;

macro_rules! impl_pattern {
    ($name:ident, $e:expr) => {
        lazy_static! {
            static ref $name: Pattern =
                Pattern::from(&one_unit_parse($e, "<akashic_record>").unwrap()).unwrap();
        }
    };
}

impl_pattern!(DEFINE_PATTERN, "('let name expr)");

impl_pattern!(FACT_PATTERN, "('fact name expr ...)");

impl_pattern!(RULE_PATTERN, "('rule params expr ...)");

impl_pattern!(RULE_PARAMS_PATTERN, "(name args ...)");

impl_pattern!(FUNCTION_CALL_PATTERN, "(name args ...)");

impl_pattern!(QUERY_PATTERN, "('define prarms expr ...)");

trait FromGast {
    type Target;
    fn from_gast(input: &GAst) -> Option<Self::Target>;
}

pub trait Loader {
    fn load(&mut self, env: &Handle<Scope>, input: &GAst) -> Option<()>;
}

impl FromGast for Expr {
    type Target = Self;

    fn from_gast(input: &GAst) -> Option<Self::Target> {
        todo!()
    }
}

fn get_value<'a>(i: &'a Expr, env: &Handle<Scope>) -> Option<Value> {
    match i {
        Expr::Value(v) => Some(v.clone()),
        Expr::Variable(k) => env.find(k),
        Expr::FunctionCall(_) => None,
    }
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
            .map(|x| Expr::from_gast(x).map(|x| get_value(&x, env)).flatten())
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
