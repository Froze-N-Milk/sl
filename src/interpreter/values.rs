use std::rc::Rc;

use crate::ast::Expression;

use super::{env::Lookup, Env, EnvEvalResult, EvalResult};

#[derive(Clone)]
pub enum Value<'env> {
	Procedure(
		Rc<dyn Fn(Env<'env>) -> EnvEvalResult<'env> + 'env>,
		Rc<[&'env str]>,
	),
	Macro(Rc<dyn Fn(Env<'env>, &[Value<'env>]) -> EnvEvalResult<'env> + 'env>),
	Symbol(&'env str),
	List(Rc<[Value<'env>]>),
	//Str(&'env str),
}

impl <'env> From<&Expression<'env>> for Value<'env> {
    fn from(value: &Expression<'env>) -> Self {
        match value {
            Expression::SExpr(expressions) => Value::List(expressions.iter().map(|e| e.into()).collect::<Vec<_>>().into()),
            Expression::Symbol(id) => Value::Symbol(id),
        }
    }
}

impl<'env> Value<'env> {
	pub fn from_env(env: Env<'env>, str: &'env str) -> EvalResult<'env> {
		match str {
			//_ if str.starts_with('"') && str.ends_with('"') => {
			//	Ok(Self::Str(&str[1..str.len()]))
			//}
			_ => Ok(env
				.lookup(&str)
				.expect(&format!("unable to find value for name \"{str}\""))
				.clone()),
		}
	}
}

impl<'env> core::fmt::Display for Value<'env> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Value::Procedure(_, args) => {
				write!(f, "procedure;")?;
				if !args.is_empty() {
					write!(f, " args: ({}", args[0])?;
					let _ = &args[1..]
						.iter()
						.try_for_each(|arg| write!(f, " {}", arg))?;
					write!(f, ")")
				} else {
					write!(f, " args: ()")
				}
			}
			Value::Macro(_) => write!(f, "macro procedure"),
			Value::Symbol(expression) => write!(f, "{expression}"),
			Value::List(lst) => {
				if !lst.is_empty() {
					write!(f, "({}", lst[0])?;
					let _ = &lst[1..]
						.iter()
						.try_for_each(|value| write!(f, " {}", value))?;
					write!(f, ")")
				} else {
					write!(f, "()")
				}
			} //Value::Str(str) => write!(f, "\"{str}\""),
		}
	}
}

impl<'env> core::fmt::Debug for Value<'env> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self}")
	}
}
