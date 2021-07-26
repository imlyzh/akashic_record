use std::collections::HashMap;

use sexpr_ir::gast::{GAst, constant::Constant, list::List, symbol::Symbol};
use sexpr_process::capture::{Capture, Catch};

use crate::structs::{rule::{Call, Expr, FactQuery, Pattern}, value::{Handle, Value}};

use super::utils::*;

pub trait FromGast {
    type Target;
    fn from_gast(input: &GAst) -> Option<Self::Target>;
}

//////////////////////////////


macro_rules! ImplCastItem {
    ($i:expr, $name:ident) => {
        if let Constant::$name(x) = $i {
            return Some(Value::$name(x.clone()));
        }
    };
}

fn simple_value_from_gast(i: &Constant) -> Option<Value> {
    if let Constant::Nil = i {
        return Some(Value::Nil);
    }
    ImplCastItem!(i, Sym);
    ImplCastItem!(i, Bool);
    ImplCastItem!(i, Char);
    ImplCastItem!(i, Int);
    ImplCastItem!(i, Uint);
    ImplCastItem!(i, Float);
    ImplCastItem!(i, Str);
    unreachable!()
}


fn symbol_from_sexpr(i: &GAst) -> Option<Handle<Symbol>> {
    i.get_const()?.get_sym()
}


/////////////////////////////


impl FromGast for Value {
    type Target = Self;

    fn from_gast(input: &GAst) -> Option<Self::Target> {
        match input {
            GAst::Const(x) => simple_value_from_gast(x),
            GAst::List(x) => if let Ok(capture) = SYMBOL_LITERIAL_PATTERN.catch(input) {
                let (cap_name, capture) = capture.first()?;
                debug_assert_eq!(cap_name.0.as_str(), "sym");
                let capture = capture.get_one().unwrap().get_const()?.get_sym()?;
                Some(Value::Sym(capture))
            } else {
                None
            },
        }
    }
}


impl FromGast for Call {
    type Target = Self;

    fn from_gast(input: &GAst) -> Option<Self::Target> {
        let capture = FACT_QUERY_PATTERN.catch(input).ok()?;
        let capture: HashMap<Handle<Symbol>, Capture> = capture.into_iter().collect();
        let call_name = capture.get(&Symbol::new("name")).unwrap()
            .get_one().unwrap()
            .get_const()?
            .get_sym()?;
        let args = capture.get(&Symbol::new("args")).unwrap().get_many().unwrap();
        let args: Option<_> = args.iter().map(Expr::from_gast).collect();
        let args = args?;
        Some(Call { call_name, args })
    }
}

impl FromGast for Expr {
    type Target = Self;

    fn from_gast(input: &GAst) -> Option<Self::Target> {
        match input {
            GAst::Const(_) => if let Some(x) = symbol_from_sexpr(input) {
                Some(Expr::Variable(x))
            } else {
                Some(Expr::Value(Value::from_gast(input)?))
            },
            GAst::List(_) => if let Ok(capture) = SYMBOL_LITERIAL_PATTERN.catch(input) {
                let (cap_name, capture) = capture.first()?;
                debug_assert_eq!(cap_name.0.as_str(), "sym");
                let capture = capture.get_one().unwrap().get_const()?.get_sym()?;
                Some(Expr::Value(Value::Sym(capture)))
            } else {
                Call::from_gast(input).map(|x| Expr::FunctionCall(Handle::new(x)))
            },
        }
    }
}


impl FromGast for Pattern {
    type Target = Self;

    fn from_gast(input: &GAst) -> Option<Self::Target> {
        match input {
            GAst::Const(c) => if let Some(sym) = symbol_from_sexpr(input) {
                if sym.0.as_str() == "_" {
                    Some(Pattern::Ignore)
                } else {
                    Some(Pattern::Variable(sym.clone()))
                }
            } else {
                Some(Pattern::Constant(simple_value_from_gast(c)?))
            },
            GAst::List(_) => if let Ok(capture) = SYMBOL_LITERIAL_PATTERN.catch(input) {
                let (cap_name, capture) = capture.first()?;
                debug_assert_eq!(cap_name.0.as_str(), "sym");
                let capture = capture.get_one().unwrap().get_const()?.get_sym()?;
                Some(Pattern::Constant(Value::Sym(capture)))
            } else if let Ok(capture) = TUPLE_PATTERN_PATTERN.catch(input) {
                let (cap_name, capture) = capture.first()?;
                debug_assert_eq!(cap_name.0.as_str(), "args");
                let capture: Option<_> = capture
                    .get_many().unwrap()
                    .iter()
                    .map(Pattern::from_gast)
                    .collect();
                let capture = capture?;
                Some(Pattern::Tuple(capture))
            } else if let Ok(capture) = LIST_HAS_EXTEND_PATTERN_PATTERN.catch(input) {
                let capture: HashMap<Handle<Symbol>, Capture> = capture.into_iter().collect();
                let args = capture.get(&Symbol::new("args")).unwrap().get_many().unwrap();
                let extend = capture.get(&Symbol::new("extend")).unwrap().get_one().unwrap();
                let args: Option<Handle<[_]>> = args.iter().map(Pattern::from_gast).collect();
                let args = args?;
                let extend = Pattern::from_gast(extend)?;
                Some(Pattern::List(args, Some(Handle::new(extend))))
            } else if let Ok(capture) = LIST_PATTERN_PATTERN.catch(input) {
                let (cap_name, capture) = capture.first()?;
                debug_assert_eq!(cap_name.0.as_str(), "args");
                let capture: Option<_> = capture
                    .get_many().unwrap()
                    .iter()
                    .map(Pattern::from_gast)
                    .collect();
                let capture = capture?;
                Some(Pattern::List(capture, None))
            } else {
                None
            },
        }
    }
}

impl FromGast for FactQuery {
    type Target = Self;

    fn from_gast(input: &GAst) -> Option<Self::Target> {
        let capture = FACT_QUERY_PATTERN.catch(input).ok()?;
        let capture: HashMap<Handle<Symbol>, Capture> = capture.into_iter().collect();
        let name = capture.get(&Symbol::new("name")).unwrap()
            .get_one().unwrap()
            .get_const()?
            .get_sym()?;
        let args = capture.get(&Symbol::new("args")).unwrap().get_many().unwrap();
        let args: Option<_> = args.iter().map(Expr::from_gast).collect();
        let args = args?;
        Some(FactQuery { name, args })
    }
}