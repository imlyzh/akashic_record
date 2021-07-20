use std::{collections::HashMap, fmt::Display};

use sexpr_ir::gast::symbol::Symbol;

pub type Handle<T> = sexpr_ir::gast::Handle<T>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Char(char),
    Uint(u64),
    Int(i64),
    Float(f64),
    Str(Handle<String>),
    Sym(Handle<Symbol>),
    Pair(Handle<Pair>),
    Tuple(Handle<Tuple>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(v) => write!(f, "{}", v),
            // Value::Char(v) => write!(f, "'{}'", v),
            Value::Uint(v) => write!(f, "{}", v),
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Str(v) => write!(f, "\"{}\"", v),
            Value::Sym(v) => write!(f, "{}", v),
            Value::Char(v) => write!(f, "(char \"{}\")", v),
            Value::Pair(v) => v.fmt(f),
            Value::Tuple(v) => v.fmt(f),
        }
    }
}

impl Display for Pair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut this = self;
        write!(f, "'(")?;
        let mut start = true;
        loop {
            match this {
                Pair(v, Value::Pair(t)) => {
                    if !start {
                        write!(f, " ")?;
                    }
                    v.fmt(f)?;
                    this = t;
                    start = false;
                    continue;
                }
                Pair(v, Value::Nil) => {
                    if !start {
                        write!(f, " ")?;
                    }
                    v.fmt(f)?;
                    break;
                }
                Pair(v, t) => {
                    if !start {
                        write!(f, " ")?;
                    }
                    v.fmt(f)?;
                    write!(f, " . ")?;
                    t.fmt(f)?;
                    break;
                }
            }
        }
        write!(f, ")")
    }
}

impl Display for Tuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = self.0.iter().map(Value::to_string).collect::<Vec<_>>();
        write!(f, "(vec {})", r.join(" "))
    }
}

impl Display for Dict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = self
            .0
            .iter()
            .map(|(k, v)| format!("'(\"{}\" . {})", k, v))
            .collect::<Vec<_>>();
        write!(f, "(dict {})", r.join(" "))
    }
}

macro_rules! impl_is_type {
    ($name:ident, $tp:ident) => {
        pub fn $name(&self) -> bool {
            matches!(self, Value::$tp(_))
        }
    };
}

impl Value {
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }
    impl_is_type!(is_bool, Bool);
    impl_is_type!(is_char, Char);
    impl_is_type!(is_int, Int);
    impl_is_type!(is_uint, Uint);
    impl_is_type!(is_float, Float);
    impl_is_type!(is_str, Str);
    impl_is_type!(is_sym, Sym);
    impl_is_type!(is_pair, Pair);
    impl_is_type!(is_dict, Tuple);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pair(pub Value, pub Value);

#[derive(Debug, Clone)]
pub struct Dict(pub HashMap<Handle<String>, Value>);

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple(pub Vec<Value>);

impl From<&[Value]> for Value {
    fn from(i: &[Value]) -> Self {
        if let Some(left) = i.first() {
            let right = Value::from(&i[1..]);
            Value::Pair(Handle::new(Pair(left.clone(), right)))
        } else {
            Value::Nil
        }
    }
}
