use std::collections::HashMap;

use sexpr_ir::gast::{symbol::Symbol, Handle};

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct FactRecord(pub HashMap<(Handle<Symbol>, usize), ValueTable>);

#[derive(Debug, Clone)]
pub struct ValueTable(pub Vec<ValueLine>);

#[derive(Debug, Clone)]
pub struct ValueLine(pub Handle<[Value]>);
