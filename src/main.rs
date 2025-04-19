mod fastpass;
mod ast;
mod interpreter;

use std::fs;

use ast::sl_parser;
use fastpass::{Parser, View};
use interpreter::interpret;

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let file = fs::read_to_string(args.next().unwrap()).unwrap();
    let parser = sl_parser();
    match parser.parse(View::new(&file)) {
        Ok((_, res)) => {
            res.iter().try_for_each(|expr| interpret(expr.clone())).unwrap();
            //res.iter().for_each(|expr| println!("expr: \n{}", expr));
            //println!("res: \"{}\"", res)
        },
        Err(err) => println!("something went wrong:\n{}", fastpass::Display(err))
    }
    //let parser = ast::atlas_parser();
    //match parser.parse(&args.next().unwrap()) {
    //    Ok((buf, res)) => {
    //        println!("remaining: \"{}\"", buf);
    //        res.iter().for_each(|expr| println!("expr: \n{}", expr));
    //    },
    //    Err(err) => println!("something went wrong: {}", err)
    //}
}

