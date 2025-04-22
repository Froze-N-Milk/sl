use core::cmp::PartialEq;
use core::fmt::Display;
use std::rc::Rc;

use super::{env, env::Lookup, Env, EvalResult};

#[derive(Clone)]
pub enum Value<'env> {
    Procedure(
        Env<'env>,
        Rc<dyn Fn(Env<'env>, &[Value<'env>]) -> EvalResult<'env> + 'env>,
        Rc<dyn Display + 'env>,
    ),
    Symbol(&'env str),
    Bool(bool),
    List(Rc<[Value<'env>]>),
}

impl<'env> PartialEq for Value<'env> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Symbol(id) => matches!(other, Value::Symbol(other_id) if id == other_id),
            Value::List(lst) => matches!(
                    other,
                    Value::List(other_lst)
                        if lst.iter().zip(other_lst.iter()).all(|(a, b)| a == b)
            ),
            Value::Procedure(_, fn_ptr, _) => matches!(
                    other,
                    Value::Procedure(_, other_fn_ptr, _)
                        if Rc::ptr_eq(&fn_ptr, &other_fn_ptr)
            ),
            Value::Bool(bool) => matches!(other, Value::Bool(other_bool) if bool == other_bool),
        }
    }
}

impl<'env> Value<'env> {
    pub fn from_env(env: Env<'env>, str: &'env str) -> EvalResult<'env> {
        Ok(env
            .clone()
            .lookup(&str)
            .expect(&format!(
                "unable to find value for name \"{str}\"\nenv:\n{:?}\n",
                env::Debug::new(env)
            ))
            .clone())
    }
}

impl<'env> core::fmt::Display for Value<'env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Procedure(_, _, repr) => write!(f, "(procedure {repr})"),
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
            }
            Value::Bool(bool) => match bool {
                true => write!(f, "#t"),
                false => write!(f, "#f"),
            },
        }
    }
}

impl<'env> core::fmt::Debug for Value<'env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
