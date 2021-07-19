use std::collections::HashMap;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::value::Value;


pub type RuleRecord = HashMap<(Handle<Symbol>, usize), RuleTable>;

pub type RuleTable = Vec<RuleBody>;


pub type Prarms = Handle<[Handle<Symbol>]>;

#[derive(Debug, Clone)]
pub struct RuleBody {
    pub prarms: Prarms,
    pub bodys: Handle<[Fact]>,
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
