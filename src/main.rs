mod fastpass;

use std::fs;

use fastpass::View;

mod frc;
mod list;
mod interpreter;
mod typed_interpreter;

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    let file = fs::read_to_string(args.next().unwrap()).unwrap();
    interpreter::interpret(View::new(&file))
}
