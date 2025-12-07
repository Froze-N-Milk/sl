use std::{fmt::Display, rc::Rc};

use crate::frc::Frc;

mod ast;

#[derive(Clone)]
pub enum LinkedList<T> {
    Cons(Frc<(T, LinkedList<T>)>),
    Tail,
}

impl<T> LinkedList<T> {
    fn any(&self, f: impl Fn(&T) -> bool) -> bool {
        self.find(f).is_some()
    }
    fn find(&self, f: impl Fn(&T) -> bool) -> Option<&T> {
        match self {
            LinkedList::Cons(frc) => match frc.as_ref() {
                (value, _) if f(&value) => Some(value),
                (_, tail) => tail.find(f),
            },
            LinkedList::Tail => None,
        }
    }
    fn first<U>(&self, f: impl Fn(&T) -> Option<U>) -> Option<U> {
        match self {
            LinkedList::Cons(frc) => {
                let (e, tail) = frc.as_ref();

                match f(e) {
                    Some(result) => Some(result),
                    None => tail.first(f),
                }
            }
            LinkedList::Tail => None,
        }
    }
}

#[derive(Clone)]
pub struct Env {
    explicit: LinkedList<(Rc<str>, Syntax)>,
    implicit: LinkedList<fn(&str, &Self) -> Option<Syntax>>,
    pub debug: bool,
    pub ident: usize,
}
impl Env {
    fn new() -> Self {
        Self {
            explicit: LinkedList::Tail,
            implicit: LinkedList::Tail,
            debug: false,
            ident: 0,
        }
    }
    fn bind(self, sym: Rc<str>, value: Syntax) -> Self {
        Self {
            explicit: LinkedList::Cons(Frc::cons(((sym, value), self.explicit))),
            ..self
        }
    }
    fn bind_str(self, sym: &str, value: Syntax) -> Self {
        self.bind(sym.into(), value)
    }
    fn bind_special(self, f: fn(&str, env: &Self) -> Option<Syntax>) -> Self {
        Self {
            implicit: LinkedList::Cons(Frc::cons((f, self.implicit))),
            ..self
        }
    }
    fn lookup(&self, sym: &str) -> Option<Syntax> {
        match self.explicit.first(|(s, v)| match **s == *sym {
            true => Some(v.clone()),
            false => None,
        }) {
            Some(value) => Some(value),
            None => self.implicit.first(|f| f(sym, self)),
        }
    }
    fn with_debug(self, debug: bool) -> Self {
        Self { debug, ..self }
    }
    fn with_scope(self, ident: usize) -> Self {
        Self {
            ident,
            ..self
        }
    }
    fn increment_scope(self) -> Self {
        Self {
            ident: self.ident + 1,
            ..self
        }
    }
}

macro_rules! include_std_lib_file {
    ($env:expr, $file:expr) => {
        $env.bind_str(
            $file,
            ast::sl(crate::fastpass::View::new(include_str!(concat!(
                "stdlib/", $file
            ))))
            .unwrap()[0]
                .clone()
                .eval(&$env),
        )
    };
}

#[derive(Clone)]
pub enum Proc {
    DefinedProc {
        arg_sym: Rc<str>,
        closure_env: Env,
        body: Frc<Syntax>,
    },

    // branching and primitive conditionals
    IfHuh,
    ConsHuh,
    BoolHuh,
    NumberHuh,
    ZeroHuh,
    NilHuh,
    SymbolHuh,
    ProcedureHuh,

    // Numbers
    Add,
    Subtract,

    // cons
    Cons,
    Car,
    Cdr,

    Eval,
    Apply,

    DefinedMacro {
        arg_sym: Rc<str>,
        closure_env: Env,
        body: Frc<Syntax>,
    },
    Macro,
    Lambda,

    Quote,
    Unquote,
    Quasiquote,
}

impl Proc {
    fn apply(self, env: &Env, arg: Syntax) -> Syntax {
        match self {
            Proc::DefinedProc {
                arg_sym,
                closure_env,
                body,
            } => body
                .decons()
                .eval(&closure_env.with_debug(env.debug).with_scope(env.ident).bind(arg_sym, arg.map(|arg| arg.eval(env)))),
            Proc::IfHuh => {
                let (cond, t, f) = arg.extract3();
                let Syntax::Bool(cond) = cond.eval(env) else {
                    todo!()
                };
                if cond {
                    t.eval(env)
                } else {
                    f.eval(env)
                }
            }
            Proc::ConsHuh => Syntax::Bool(matches!(arg.extract1().eval(env), Syntax::Cons(_))),
            Proc::BoolHuh => Syntax::Bool(matches!(arg.extract1().eval(env), Syntax::Bool(_))),
            Proc::NumberHuh => Syntax::Bool(matches!(arg.extract1().eval(env), Syntax::Number(_))),
            Proc::ZeroHuh => Syntax::Bool(matches!(arg.extract1().eval(env), Syntax::Number(0.0))),
            Proc::NilHuh => Syntax::Bool(matches!(arg.extract1().eval(env), Syntax::Nil)),
            Proc::SymbolHuh => Syntax::Bool(matches!(arg.extract1().eval(env), Syntax::Sym(_))),
            Proc::ProcedureHuh => Syntax::Bool(matches!(arg.extract1().eval(env), Syntax::Proc(_))),
            Proc::Add => arg.reduce(|l, r| {
                let Syntax::Number(l) = l.eval(env) else {
                    panic!()
                };
                let Syntax::Number(r) = r.eval(env) else {
                    panic!()
                };
                Syntax::Number(l + r)
            }),
            Proc::Subtract => arg.reduce(|l, r| {
                let Syntax::Number(l) = l.eval(env) else {
                    panic!()
                };
                let Syntax::Number(r) = r.eval(env) else {
                    panic!()
                };
                Syntax::Number(l - r)
            }),
            Proc::Cons => {
                let (car, cdr) = arg.extract2();
                Syntax::Cons(Frc::cons((car.eval(env), cdr.eval(env))))
            }
            Proc::Car => match arg.extract1().eval(env) {
                Syntax::Cons(cons) => cons.decons().0,
                expr => todo!("unexpected value {}", expr),
            },
            Proc::Cdr => match arg.extract1().eval(env) {
                Syntax::Cons(cons) => cons.decons().1,
                _ => todo!(),
            },
            Proc::Eval => arg.extract1().eval(env),
            Proc::Apply => {
                let (proc, args) = arg.extract2();
                match proc.eval(env) {
                    Syntax::Proc(proc) => proc.apply(env, args.eval(env).map(|a| Syntax::Cons(Frc::cons((Syntax::Proc(Proc::Quote), Syntax::Cons(Frc::cons((a, Syntax::Nil)))))))),
                    non_proc => panic!("cannot apply non-procedure {}", non_proc),
                }
            }
            Proc::DefinedMacro {
                arg_sym,
                closure_env,
                body,
            } => body.decons().eval(&closure_env.with_debug(env.debug).with_scope(env.ident).bind(arg_sym, arg)),
            Proc::Macro => {
                let (arg, body) = arg.extract2();
                let Syntax::Sym(arg) = arg else { panic!() };
                Syntax::Proc(Proc::DefinedMacro {
                    arg_sym: arg,
                    closure_env: env.clone(),
                    body: Frc::cons(body),
                })
            }
            Proc::Lambda => {
                let (arg, body) = arg.extract2();
                let Syntax::Sym(arg) = arg else { panic!() };
                Syntax::Proc(Proc::DefinedProc {
                    arg_sym: arg,
                    closure_env: env.clone(),
                    body: Frc::cons(body),
                })
            }
            Proc::Quote => arg.extract1(),
            // NOTE: can't be run directly
            Proc::Unquote => panic!(),
            Proc::Quasiquote => quasiquote(arg.extract1(), env),
        }
    }
}

#[derive(Clone)]
pub enum Syntax {
    Sym(Rc<str>),
    Nil,
    Bool(bool),
    Number(f64),
    Cons(Frc<(Self, Self)>),
    Proc(Proc),
}

impl Default for Syntax {
    fn default() -> Self {
        Self::Nil
    }
}

fn display_list(cons: &(Syntax, Syntax), f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &cons.1 {
        Syntax::Cons(frc) => {
            write!(f, "{} ", cons.0)?;
            display_list(frc.as_ref(), f)
        }
        Syntax::Nil => write!(f, "{})", cons.0),
        _ => write!(f, "{} . {})", cons.0, cons.1),
    }
}

impl Display for Syntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Syntax::Sym(s) => write!(f, "'{}", s),
            Syntax::Nil => write!(f, "'()"),
            Syntax::Bool(b) => {
                if *b {
                    write!(f, "#t")
                } else {
                    write!(f, "#f")
                }
            }
            Syntax::Number(n) => write!(f, "{}", n),
            Syntax::Cons(frc) => {
                write!(f, "(")?;
                display_list(frc.as_ref(), f)
            }
            Syntax::Proc(proc) => match proc {
                Proc::DefinedProc {
                    arg_sym,
                    closure_env: _,
                    body,
                } => write!(f, "(lambda {} {})", arg_sym, body.as_ref()),
                Proc::IfHuh => write!(f, "if?"),
                Proc::ConsHuh => write!(f, "cons?"),
                Proc::BoolHuh => write!(f, "bool?"),
                Proc::NumberHuh => write!(f, "number?"),
                Proc::ZeroHuh => write!(f, "0?"),
                Proc::NilHuh => write!(f, "nil?"),
                Proc::SymbolHuh => write!(f, "sym?"),
                Proc::ProcedureHuh => write!(f, "proc?"),
                Proc::Add => write!(f, "+"),
                Proc::Subtract => write!(f, "-"),
                Proc::Cons => write!(f, "cons"),
                Proc::Car => write!(f, "car"),
                Proc::Cdr => write!(f, "cdr"),
                Proc::Eval => write!(f, "eval"),
                Proc::Apply => write!(f, "apply"),
                Proc::DefinedMacro {
                    arg_sym,
                    closure_env: _,
                    body,
                } => write!(f, "(macro {} {})", arg_sym, body.as_ref()),
                Proc::Macro => write!(f, "macro"),
                Proc::Lambda => write!(f, "lambda"),
                Proc::Quote => write!(f, "quote"),
                Proc::Unquote => write!(f, "unquote"),
                Proc::Quasiquote => write!(f, "quasiquote"),
            },
        }
    }
}

impl Syntax {
    pub fn eval(self, env: &Env) -> Self {
        if env.debug { eprintln!("{}evaluating: {}", "│ ".repeat(env.ident), self); }
        let res = match self {
            Syntax::Sym(sym) => match env.lookup(&sym) {
                Some(v) => v,
                None => panic!("unbound symbol: {}", sym),
            },
            Syntax::Cons(frc) => {
                let (car, cdr) = frc.decons();
                let env = &env.clone().increment_scope();
                match car.eval(env) {
                    Syntax::Proc(proc) => proc.apply(env, cdr),
                    non_proc => panic!("cannot invoke non-procedure {}", non_proc),
                }
            }
            _ => self,
        };
        if env.debug { eprintln!("{}evaluated to: {}", "│ ".repeat(env.ident), res); }
        res
    }

    fn map(self, f: impl Fn(Self) -> Self) -> Self {
        match self {
            Syntax::Nil => self,
            Syntax::Cons(cons) => Syntax::Cons(cons.map(|(car, cdr)| (f(car), cdr.map(f)))),
            non_list => panic!("cannot map non-list {}", non_list),
        }
    }

    fn fold(self, initial: Self, f: impl Fn(Self, Self) -> Self) -> Self {
        match self {
            Syntax::Nil => initial,
            Syntax::Cons(cons) => {
                let (car, cdr) = cons.decons();
                cdr.fold(f(initial, car), f)
            }
            _ => panic!(),
        }
    }

    fn reduce(self, f: impl Fn(Self, Self) -> Self) -> Self {
        match self {
            Syntax::Nil => self,
            Syntax::Cons(cons) => {
                let (car, cdr) = cons.decons();
                cdr.fold(car, f)
            }
            _ => todo!(),
        }
    }

    fn extract1(self) -> Self {
        match self {
            Syntax::Cons(cons) => match cons.decons() {
                (car, Syntax::Nil) => car,
                (car, cdr) => todo!("incorrect arg: ({} {})", car, cdr),
            },
            _ => todo!(),
        }
    }

    fn extract2(self) -> (Self, Self) {
        match self {
            Syntax::Cons(cons) => match cons.decons() {
                (car, cons @ Syntax::Cons(_)) => (car, cons.extract1()),
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    fn extract3(self) -> (Self, Self, Self) {
        match self {
            Syntax::Cons(cons) => match cons.decons() {
                (car, cons @ Syntax::Cons(_)) => {
                    let tail = cons.extract2();
                    (car, tail.0, tail.1)
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}

fn quasiquote(expr: Syntax, env: &Env) -> Syntax {
    match expr {
        Syntax::Cons(cons) => match cons.as_ref().0 {
            Syntax::Proc(Proc::Unquote) => cons.decons().1.extract1().eval(env),
            _ => Syntax::Cons(cons.map(|(car, cdr)| (quasiquote(car, env), quasiquote(cdr, env)))),
        },
        Syntax::Sym(sym) => match sym.starts_with(',') {
            true => match env.lookup(&sym[1..]) {
                Some(v) => v,
                None => panic!("unbound symbol: {}", &sym[1..]),
            },
            false => Syntax::Sym(sym),
        }
        _ => expr,
    }
}

fn make_env(debug: bool) -> Env {
    Env::new()
        .with_debug(debug)
        .bind_special(|sym, _| match sym.parse::<f64>() {
            Ok(number) => Some(Syntax::Number(number)),
            Err(_) => None,
        })
        .bind_special(|sym, _| match sym.starts_with('\'') {
            true => Some(Syntax::Sym(Rc::from(&sym[1..]))),
            false => None,
        })
        .bind_special(|sym, _| match sym.starts_with('`') {
            true => Some(Syntax::Sym(Rc::from(&sym[1..]))),
            false => None,
        })
        .bind_str("#t", Syntax::Bool(true))
        .bind_str("#f", Syntax::Bool(false))
        .bind_str("macro", Syntax::Proc(Proc::Macro))
        .bind_str("lambda", Syntax::Proc(Proc::Lambda))
        .bind_str("quote", Syntax::Proc(Proc::Quote))
        .bind_str("'", Syntax::Proc(Proc::Quote))
        .bind_str("unquote", Syntax::Proc(Proc::Unquote))
        .bind_str(",", Syntax::Proc(Proc::Unquote))
        .bind_str("quasiquote", Syntax::Proc(Proc::Unquote))
        .bind_str("`", Syntax::Proc(Proc::Quasiquote))
        .bind_str("if?", Syntax::Proc(Proc::IfHuh))
        .bind_str("cons?", Syntax::Proc(Proc::ConsHuh))
        .bind_str("bool?", Syntax::Proc(Proc::BoolHuh))
        .bind_str("number?", Syntax::Proc(Proc::NumberHuh))
        .bind_str("0?", Syntax::Proc(Proc::ZeroHuh))
        .bind_str("nil?", Syntax::Proc(Proc::NilHuh))
        .bind_str("symbol?", Syntax::Proc(Proc::SymbolHuh))
        .bind_str("proc?", Syntax::Proc(Proc::ProcedureHuh))
        .bind_str("+", Syntax::Proc(Proc::Add))
        .bind_str("-", Syntax::Proc(Proc::Subtract))
        .bind_str("cons", Syntax::Proc(Proc::Cons))
        .bind_str("car", Syntax::Proc(Proc::Car))
        .bind_str("cdr", Syntax::Proc(Proc::Cdr))
        .bind_str("eval", Syntax::Proc(Proc::Eval))
        .bind_str("apply", Syntax::Proc(Proc::Apply))
}

fn include_std(env: Env) -> Env {
    let env = include_std_lib_file!(env.clone(), "fix");
    let env = include_std_lib_file!(env.clone(), "list:k");
    let env = include_std_lib_file!(env.clone(), "list");
    env
}

pub fn interpret<'buf>(buf: crate::fastpass::View<'buf>) {
    let env = make_env(false);
    let env = include_std(env).with_debug(true);

    let parsed = ast::sl(buf).unwrap();
    parsed
        .into_iter()
        .for_each(|expr| println!("{}", expr.clone().eval(&env)));
}
