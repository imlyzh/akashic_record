use std::collections::HashMap;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::value::Value;


pub type RuleRecord = HashMap<(Handle<Symbol>, usize), RuleTable>;

pub type RuleTable = Vec<RuleBody>;


#[derive(Debug, Clone)]
pub struct RuleBody {
    pub prarms: Prarms,
    pub bodys: Handle<[Fact]>,
}


pub type Prarms = Handle<[Pattern]>;


#[derive(Debug, Clone)]
pub enum Pattern {
    Ignore,
    Variable(Handle<Symbol>),
    Constant(Value),
    Tuple(Vec<Pattern>),
    List(Vec<Pattern>, Option<Handle<Pattern>>),
}


#[derive(Debug, Clone)]
pub struct Fact {
    pub call_name: Handle<Symbol>,
    pub args: Vec<Expr>,
}


#[derive(Debug, Clone)]
pub enum Expr {
    Value(Value),
    Variable(Handle<Symbol>),
    FunctionCall(Handle<Call>),
}

#[derive(Debug, Clone)]
pub struct Call(pub Vec<Expr>);
