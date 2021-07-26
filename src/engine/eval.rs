use sexpr_ir::gast::{symbol::Symbol, Handle};

use crate::structs::{rule::Expr, scope::Scope, value::Value};

fn eval_function(name: &Handle<Symbol>, args: &[Value]) -> Value {
    todo!()
}

pub fn eval_value(i: &Expr, env: &Handle<Scope>) -> Option<Value> {
    match i {
        Expr::Value(v) => Some(v.clone()),
        Expr::Variable(k) => env.find(k),
        Expr::FunctionCall(c) => {
            let r: Option<Vec<Value>> = c.args.iter().map(|x| eval_value(x, env)).collect();
            let r = r?;
            Some(eval_function(&c.call_name, &r))
        }
    }
}
