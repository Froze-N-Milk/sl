use std::rc::Rc;

use crate::ast::Expression;

mod env;
mod inbuilt;
mod values;

use values::Value;

type EvalResult<'env> = Result<Value<'env>, ()>;
type Env<'env> = env::NameEnv<'env, Value<'env>>;
type EnvEvalResult<'env> = Result<(Env<'env>, Value<'env>), ()>;

pub fn interpret<'env>(expr: Expression<'env>) -> Result<(), ()> {
	let env = Env::new()
		// TODO, define basic functions
		.bind(env::Values::new([
			env::Value("lambda", Value::Macro(Rc::new(inbuilt::lambda))),
			env::Value("macro", Value::Macro(Rc::new(inbuilt::lambda_macro))),
			env::Value("let", Value::Macro(Rc::new(inbuilt::bind_let))),
			env::Value("quote", Value::Macro(Rc::new(inbuilt::quote))),
			env::Value("quasiquote", Value::Macro(Rc::new(inbuilt::quasiquote))),
			env::Value("eval", inbuilt::embed_eval()),
		]));

	let (_, value) = eval(env, Value::from(&expr))?;
	println!("{value}");
	Ok(())
}

fn eval<'env>(env: Env<'env>, expr: Value<'env>) -> EnvEvalResult<'env> {
	match expr {
		Value::List(expressions) => invoke(env, &expressions),
		Value::Symbol(str) => Ok((env.clone(), Value::from_env(env, str)?)),
		_ => Ok((env, expr)),
	}
}

fn invoke<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EnvEvalResult<'env> {
	match exprs {
		[] => panic!("cannot eval {exprs:#?}"),
		// TODO: match procedure type first?, there are procedures that should interpret
		// the args differently
		[procedure, args @ ..] => {
			let (_, procedure) = eval(env.clone(), procedure.clone())?;
			match procedure {
				Value::Procedure(procedure, args_bindings) => {
					// TODO better pattern matching
					if args_bindings.len() != args.len() {
						panic!("wrong number of args.\nbindings: {args_bindings:#?}\nargs: {args:#?}");
					}

					let env = args_bindings.iter().zip(args.iter()).try_fold(
						env,
						|env, (name, expression)| {
							let (env, val) =
								eval(env, expression.clone())?;
							Ok(env.bind(env::Value(*name, val)))
						},
					)?;

					procedure(env)
				}
				Value::Macro(macro_procedure) => macro_procedure(env, args),
				_ => panic!("cannot call non-procedure: {procedure}"),
			}
		}
	}
}
