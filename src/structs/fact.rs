use std::collections::HashMap;

use sexpr_ir::gast::{symbol::Symbol, Handle};

use super::value::Value;

#[derive(Debug, Default, Clone)]
pub struct FactRecord(pub HashMap<(Handle<Symbol>, usize), ValueTable>);

#[derive(Debug, Clone)]
pub struct ValueTable(pub Vec<ValueLine>);

#[derive(Debug, Clone)]
pub struct ValueLine(pub Handle<[Value]>);
