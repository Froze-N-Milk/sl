mod fastpass;
mod ast;
mod interpreter;

use std::fs;

use ast::sl;
use fastpass::View;
use interpreter::interpret;

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let file = fs::read_to_string(args.next().unwrap()).unwrap();
    match sl(View::new(&file)) {
        Ok(expr) => interpret(expr).unwrap(),
        Err(err) => println!("something went wrong:\n{}", fastpass::Display(err))
    };
}

