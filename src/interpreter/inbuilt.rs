use std::rc::Rc;

use crate::interpreter::{eval, Value};

use super::{env, DisplayList, Env, EvalResult};

pub fn lambda<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(
        env,
        Rc::new(|env, exprs| match exprs {
            [Value::List(bindings), body] => {
                Ok(lambda_internal(env.clone(), bindings, body.clone()))
            }
            _ => panic!("{exprs:#?} did not match any forms of macro procedure \"lambda\""),
        }),
        Rc::new("(bindings...) body"),
    )
}

fn lambda_internal<'env>(
    env: Env<'env>,
    bindings: &[Value<'env>],
    body: Value<'env>,
) -> Value<'env> {
    let bindings: Rc<[&'env str]> = bindings
        .iter()
        .map(|v| match v {
            Value::Symbol(binding) => *binding,
            _ => panic!("invalid binding: {v}"),
        })
        .collect::<Vec<_>>()
        .into();
    let proc_bindings = bindings.clone();
    let procedure = Rc::new(move |env: Env<'env>, args: &[Value<'env>]| {
        let bindings = proc_bindings.clone();
        if args.len() != bindings.len() {
            panic!("wrong number of args:\nbindings: {bindings:#?}\nargs: {args:#?}");
        }
        let mut v = Vec::with_capacity(args.len());
        bindings
            .iter()
            .zip(args.iter())
            .try_for_each(|(binding, arg)| {
                v.push(env::Value(*binding, eval(env.clone(), arg.clone())?));
                Ok::<(), ()>(())
            })?;
        eval(env.bind(env::Values::new(Rc::from(v))), body.clone())
    });

    Value::Procedure(env, procedure, Rc::new(DisplayList(bindings.clone())))
}

pub fn lambda_macro<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(
        env,
        Rc::new(|env, exprs| match exprs {
            [Value::Symbol(binding), body] => {
                Ok(lambda_macro_internal(env.clone(), binding, body.clone()))
            }
            _ => panic!("{exprs:#?} did not match any forms of macro procedure \"macro\""),
        }),
        Rc::new("binding body"),
    )
}

fn lambda_macro_internal<'env>(
    env: Env<'env>,
    binding: &'env str,
    body: Value<'env>,
) -> Value<'env> {
    let procedure = Rc::new(move |env: Env<'env>, args: &[Value<'env>]| {
        eval(env.bind(env::Value(binding, Value::List(Rc::from(args)))), body.clone())
    });

    Value::Procedure(env, procedure, Rc::new(binding))
}

pub fn bind_let<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(
        env,
        Rc::new(|env, exprs| match exprs {
            [Value::List(bindings), body] => {
                let mut v = Vec::with_capacity(bindings.len());
                bindings.iter().try_for_each(|binding| match binding {
                    Value::List(binding) => match &binding[..] {
                        [Value::Symbol(name), expr] => {
                            v.push(env::Value(*name, eval(env.clone(), expr.clone())?));
                            Ok(())
                        }
                        _ => panic!("{binding:#?} did not match the (name value) form"),
                    },
                    _ => panic!("{binding:#?} did not match the ((name value)...) form"),
                })?;
                eval(
                    env.bind(env::Values::new(v.into_boxed_slice())),
                    body.clone(),
                )
            }
            _ => panic!("{exprs:#?} did not match any forms of macro-procedure \"let\""),
        }),
        Rc::new("((binding value)...) body"),
    )
}

pub fn quote<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(env, Rc::new(quote_internal), Rc::new("symbol"))
}

fn quote_internal<'env>(_: Env<'env>, exprs: &[Value<'env>]) -> EvalResult<'env> {
    match exprs {
        [expr] => Ok(expr.clone()),
        _ => Ok(Value::List(Rc::from(exprs))),
    }
}

pub fn quasiquote<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(env, Rc::new(quasiquote_internal), Rc::new("symbol"))
}

fn quasiquote_internal<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EvalResult<'env> {
    match exprs {
        [Value::List(exprs)] => match exprs.as_ref() {
            [Value::Symbol("unquote"), exprs] => unquote(env, &[exprs.clone()]),
            _ => {
                let mut v = Vec::with_capacity(exprs.len());
                exprs.iter().try_for_each(|expr| {
                    v.push(quasiquote_internal(env.clone(), &[expr.clone()])?);
                    Ok::<(), ()>(())
                })?;
                match &v[..] {
                    [value] => Ok(value.clone()),
                    _ => Ok(Value::List(Rc::from(v))),
                }
            }
        },
        [symbol] => quote_internal(env, &[symbol.clone()]),
        _ => panic!("{exprs:#?} did not match any forms of macro procedure \"quasiquote\""),
    }
}

fn unquote<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EvalResult<'env> {
    match exprs.as_ref() {
        [expr] => eval(env, expr.clone()),
        _ => panic!("{exprs:#?} did not match any forms of macro procedure \"unquote\""),
    }
}

pub fn embed_eval<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(
        env,
        Rc::new(|env, exprs| match exprs {
            [expr] => eval(env, expr.clone()),
            _ => panic!("{exprs:#?} did not match any forms of procedure \"eval\""),
        }),
        Rc::new("symbol"),
    )
}

pub fn begin<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(
        env,
        Rc::new(begin_internal),
        Rc::new("((define-form)...) body"),
    )
}

pub fn begin_internal<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EvalResult<'env> {
    match exprs {
        [defines @ .., body] => {
            let env = defines
                .iter()
                .try_fold(env, |env, define_expr| match define_expr {
                    Value::List(lst) => define(env, &lst),
                    _ => panic!("expected a form of \"define\" / \"define-macro\""),
                })?;
            eval(env, body.clone())
        }
        _ => panic!("{exprs:#?} did not match any forms of macro procedure \"begin\""),
    }
}

fn define<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> Result<Env<'env>, ()> {
    match exprs {
        [Value::Symbol("define"), Value::Symbol(name), value] => Ok(env
            .clone()
            .bind(env::Value(*name, eval(env, value.clone())?))),
        [Value::Symbol("define"), name_args, body] => match name_args {
            Value::List(name_args) => match name_args.as_ref() {
                [Value::Symbol(name), args @ ..] => {
                    let value = lambda_internal(env.clone(), args, body.clone());
                    Ok(env.clone().bind(env::Value(*name, value)))
                }
                _ => panic!(
                    "{exprs:#?} did not match the define form \"(define (name args...) body)\""
                ),
            },
            _ => {
                panic!("{exprs:#?} did not match the define form \"(define (name args...) body)\"")
            }
        },
        [Value::Symbol("define-macro"), name_arg, body] => {
            match name_arg {
                Value::List(name_args) => match name_args.as_ref() {
                    [Value::Symbol(name), Value::Symbol(binding)] => {
                        let value = lambda_macro_internal(env.clone(), binding, body.clone());
                        Ok(env.clone().bind(env::Value(*name, value)))
                    }
                    _ => panic!(
                    "{exprs:#?} did not match the define form \"(define-macro (name arg) body)\""
                ),
                },
                _ => {
                    panic!("{exprs:#?} did not match the define form \"(define-macro (name arg) body)\"")
                }
            }
        }
        _ => panic!(
            "{exprs:#?} did not match any forms of macro procedure \"define\" / \"define-macro\""
        ),
    }
}

fn truthy<'env>(env: Env<'env>, cond: Value<'env>) -> Result<bool, ()> {
    Ok(match eval(env.clone(), cond.clone())? {
        Value::Bool(cond) => cond,
        Value::List(exprs) => exprs.len() != 0,
        _ => true,
    })
}

pub fn if_cond<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(
        env,
        Rc::new(|env, exprs| match exprs {
            [cond, pass, fail] => {
                if truthy(env.clone(), cond.clone())? {
                    eval(env, pass.clone())
                } else {
                    eval(env, fail.clone())
                }
            }
            _ => panic!("{exprs:#?} did not match any forms of macro procedure \"if?\""),
        }),
        Rc::new("cond pass-body fail-body"),
    )
}

pub fn guard<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(
        env,
        Rc::new(|env, exprs| match exprs {
            [branches @ .., fail] => {
                for branch in branches {
                    match branch {
                        Value::List(exprs) => match guard_branch(env.clone(), exprs) {
                            Some(res) => return res,
                            _ => (),
                        },
                        _ => panic!(
                            "{exprs:#?} did not match any forms of macro procedure \"guard?\""
                        ),
                    };
                }
                eval(env, fail.clone())
            }
            _ => panic!("{exprs:#?} did not match any forms of macro procedure \"guard?\""),
        }),
        Rc::new("(guard? body)... fail"),
    )
}

fn guard_branch<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> Option<EvalResult<'env>> {
    match exprs {
        [cond, body] => match truthy(env.clone(), cond.clone()) {
            Ok(cond) => {
                if cond {
                    Some(eval(env, body.clone()))
                } else {
                    None
                }
            }
            Err(err) => Some(Err(err)),
        },
        _ => panic!(
            "{exprs:#?} did not match the (guard? body) form of macro procedure \"guard?\"'s branches"
        ),
    }
}

pub fn pmatch<'env>(env: Env<'env>) -> Value<'env> {
    Value::Procedure(
        env,
        Rc::new(|env, exprs| match exprs {
            [value, branches @ .., fail] => {
                let value = eval(env.clone(), value.clone())?;
                for branch in branches.as_ref() {
                    match branch {
                        Value::List(branch) => match branch.as_ref() {
                            [structure, body] => {
                                match structure_match(env.clone(), value.clone(), structure.clone())
                                {
                                    Ok(env) => return eval(env, body.clone()),
                                    _ => (),
                                }
                            }
                            [structure, guard, body] => {
                                match structure_match(env.clone(), value.clone(), structure.clone())
                                {
                                    Ok(env) => match truthy(env.clone(), guard.clone()) {
                                        Ok(guard) => {
                                            if guard {
                                                return eval(env, body.clone());
                                            }
                                        }
                                        _ => (),
                                    },
                                    _ => (),
                                }
                            }
                            _ => panic!(),
                        },
                        _ => panic!(
                            "{exprs:#?} did not match any forms of macro procedure \"guard?\""
                        ),
                    };
                }
                eval(env, fail.clone())
            }
            _ => panic!("{exprs:#?} did not match any forms of macro procedure \"guard?\""),
        }),
        Rc::new("(structure [guard?] body)... fail"),
    )
}

fn structure_match<'env>(env: Env<'env>, l: Value<'env>, r: Value<'env>) -> Result<Env<'env>, ()> {
    match r {
        Value::Symbol(id) => Ok(env.bind(env::Value(id, l))),
        Value::List(r) => match r.as_ref() {
            [Value::Symbol("quote"), r] => {
                if l == *r {
                    Ok(env)
                } else {
                    Err(())
                }
            }
            [Value::Symbol("quote"), ..] => panic!("invalid quote form in pmatch"),
            _ => match l {
                Value::List(l) => l.iter().zip(r.iter()).try_fold(env, |env, (l, r)| {
                    structure_match(env, l.clone(), r.clone())
                }),
                _ => Err(()),
            },
        },
        _ => {
            if l == r {
                Ok(env)
            } else {
                Err(())
            }
        }
    }
}
