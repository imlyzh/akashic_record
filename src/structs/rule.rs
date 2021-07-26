use std::collections::HashMap;

use sexpr_ir::gast::{symbol::Symbol, Handle};

use super::value::Value;

#[derive(Debug, Default, Clone)]
pub struct RuleRecord(pub HashMap<(Handle<Symbol>, usize), RuleTable>);

#[derive(Debug, Default, Clone)]
pub struct RuleTable(pub Vec<RuleBody>);

#[derive(Debug, Clone)]
pub struct RuleBody {
    pub prarms: Prarms,
    pub bodys: Handle<[FactQuery]>,
}

pub type Prarms = Handle<[Pattern]>;

#[derive(Debug, Clone)]
pub enum Pattern {
    Ignore,
    Variable(Handle<Symbol>),
    Constant(Value),
    Tuple(Handle<[Pattern]>),
    List(Handle<[Pattern]>, Option<Handle<Pattern>>),
}

#[derive(Debug, Clone)]
pub struct FactQuery {
    pub name: Handle<Symbol>,
    pub args: Handle<[Expr]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(Value),
    Variable(Handle<Symbol>),
    FunctionCall(Handle<Call>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub call_name: Handle<Symbol>,
    pub args: Box<[Expr]>,
}
