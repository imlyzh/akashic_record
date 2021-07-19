
use std::collections::HashMap;

use sexpr_ir::gast::{Handle, symbol::Symbol};

use crate::value::Value;



pub type FactRecord = HashMap<(Handle<Symbol>, usize), ValueTable>;

pub type ValueTable = Vec<ValueLine>;

pub type ValueLine = Handle<[Value]>;
