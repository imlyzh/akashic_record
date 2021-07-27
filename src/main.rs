mod engine;
mod structs;

use std::io::{stdin, stdout, Write};

use sexpr_ir::syntax::sexpr::one_unit_parse;
use structs::{scope::Scope, value::Handle};

use crate::engine::environment::Database;
use crate::engine::load::repl_eval;

fn start_repl(env: &Handle<Database>, scope: &Handle<Scope>) {
    loop {
        print!(">>> ");
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let buf = buf.trim();
        if buf.is_empty() {
            continue;
        }
        let r = one_unit_parse(buf, "<akashic_record>");
        if let Err(e) = r {
            println!("syntax err: {}", e);
            continue;
        }
        let input = r.unwrap();

        if let Some(rs) = repl_eval(env, scope, &input) {
            if let Some(x) = rs {
                for (k, v) in x.0.read().unwrap().iter() {
                    println!("{}: {}", k.0, v);
                }
            }
            println!("ok.");
        } else {
            println!("err.");
        }
    }
}

fn main() {
    println!("Welcome to Akashic Record!");
    let env = Handle::new(Database::default());
    let scope = Handle::new(Scope::default());
    start_repl(&env, &scope);
}
