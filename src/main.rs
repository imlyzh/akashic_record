mod engine;
mod structs;


use std::io::{Write, stdin, stdout};


use sexpr_ir::syntax::sexpr::one_unit_parse;
use structs::{scope::Scope, value::Handle};

use crate::engine::environment::Database;
use crate::engine::load::Loader;


fn start_repl(env: &mut Database, scope: &Handle<Scope>) {
    loop {
        print!(">> ");
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let buf = buf.trim();
        if buf.is_empty() {
            continue;
        }
        let r = one_unit_parse(buf, "<akashic_record>");
        if let Err(e) = r {
            println!("err: {}", e);
            continue;
        }
        let input = r.unwrap();
        if let Some(()) = env.load(scope, &input) {
            println!("ok.");
        } else {
            println!("err.");
        }
        println!("env: {:?}", env);
        println!("scope: {:?}", scope);
    }
}

fn main() {
    println!("Welcome to Akashic Record!");
    let mut env = Database::default();
    let scope = Handle::new(Scope::default());
    start_repl(&mut env, &scope);
}
