use std::rc::Rc;

use crate::interpreter::{eval, Value};

use super::{env, Env, EnvEvalResult, EvalResult};

pub fn lambda<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EnvEvalResult<'env> {
	match exprs {
		[Value::List(args), body] => lambda_internal(args, body.clone()).map(|v| (env, v)),
		_ => panic!("{exprs:#?} did not match any forms of macro procedure \"lambda\""),
	}
}

fn lambda_internal<'env>(args: &[Value<'env>], body: Value<'env>) -> EvalResult<'env> {
	let mut v = Vec::with_capacity(args.len());
	args.iter().try_for_each(|arg| match arg {
		Value::Symbol(name) => {
			v.push(*name);
			Ok(())
		}
		_ => panic!("invalid binding symbol: {arg}"),
	})?;

	let procedure = Rc::new(move |env| eval(env, body.clone()));
	Ok(Value::Procedure(procedure, Rc::from(v)))
}

pub fn lambda_macro<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EnvEvalResult<'env> {
	match exprs {
		[Value::Symbol(arg_name), body] => {
			let body = body.clone();
			let arg_name = *arg_name;
			let procedure = Rc::new(move |env: Env<'env>, exprs: &[Value<'env>]| {
				eval(
					env.bind(env::Value(
						arg_name,
						Value::List(Rc::from(exprs)),
					)),
					body.clone(),
				)
			});
			Ok((env, Value::Macro(procedure)))
		}
		_ => panic!("{exprs:#?} did not match any forms of macro procedure \"macro\""),
	}
}

pub fn bind_let<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EnvEvalResult<'env> {
	match exprs {
		[Value::List(bindings), body] => {
			let mut v = Vec::with_capacity(bindings.len());
			bindings.iter().try_for_each(|binding| match binding {
				Value::List(binding) => match &binding[..] {
					[Value::Symbol(name), expr] => {
						v.push(env::Value(
							*name,
							eval(env.clone(), expr.clone())
								.map(|(_, value)| value)?,
						));
						Ok(())
					}
					_ => panic!(
						"{binding:#?} did not match the (name value) form"
					),
				},
				_ => panic!("{binding:#?} did not match the ((name value)...) form"),
			})?;
			eval(
				env.bind(env::Values::new(v.into_boxed_slice())),
				body.clone(),
			)
		}
		_ => Err(()),
	}
}

pub fn quote<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EnvEvalResult<'env> {
	match exprs {
		[expr] => Ok((env, expr.clone())),
		_ => Ok((env, Value::List(Rc::from(exprs)))),
	}
}

pub fn quasiquote<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EnvEvalResult<'env> {
	match exprs {
		[Value::List(exprs)] => match exprs.as_ref() {
			[Value::Symbol("unquote"), exprs] => unquote(env, &[exprs.clone()]),
			_ => {
				let mut v = Vec::with_capacity(exprs.len());
				exprs.iter().try_for_each(|expr| {
					v.push(quasiquote(env.clone(), &[expr.clone()])
						.map(|(_, v)| v)?);
					Ok::<(), ()>(())
				})?;
				match &v[..] {
					[value] => Ok((env, value.clone())),
					_ => Ok((env, Value::List(Rc::from(v)))),
				}
			}
		},
		[symbol] => quote(env, &[symbol.clone()]),
		_ => panic!("{exprs:#?} did not match any forms of macro procedure \"quasiquote\""),
	}
}

fn unquote<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EnvEvalResult<'env> {
	match exprs.as_ref() {
		[expr] => eval(env, expr.clone()),
		_ => panic!("{exprs:#?} did not match any forms of macro procedure \"unquote\""),
	}
}

pub fn embed_eval<'env>() -> Value<'env> {
	Value::Procedure(
		Rc::new(|env| eval(env.clone(), Value::from_env(env, "value")?)),
		Rc::new(["value"]),
	)
}
