use core::fmt::Display;
use std::rc::Rc;

mod env;
mod inbuilt;
mod values;

//mod cps;

pub use values::Value;

type EvalResult<'env> = Result<Value<'env>, ()>;
type Env<'env> = env::NameEnv<'env, Value<'env>>;

pub struct DisplayList<D: Display>(Rc<[D]>);
impl<D: Display> Display for DisplayList<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "(")?;
        match self.0.as_ref() {
            [x, xs @ ..] => {
                write!(f, "{x}")?;
                for x in xs {
                    write!(f, " {x}")?;
                }
                write!(f, ")")
            }
            [] => write!(f, ")"),
        }
    }
}

pub fn interpret<'env>(expr: Value<'env>) -> Result<(), ()> {
    let env = Env::new();
    let env = env.clone().bind(env::Values::new([
        env::Value(
            "lambda",
            inbuilt::lambda(env.clone()),
        ),
        //env::Value(
        //    "macro",
        //    Value::Procedure(env.clone(), Rc::new(inbuilt::lambda_macro)),
        //),
        env::Value(
            "begin",
            inbuilt::begin(env.clone())
        ),
        env::Value(
            "let",
            inbuilt::bind_let(env.clone()),
        ),
        env::Value(
            "quote",
            inbuilt::quote(env.clone()),
        ),
        env::Value(
            "quasiquote",
            inbuilt::quasiquote(env.clone()),
        ),
        env::Value(
            "guard?",
            inbuilt::guard(env.clone())
        ),
        env::Value(
            "pmatch?",
            inbuilt::pmatch(env.clone())
        ),
        env::Value(
            "if?",
            inbuilt::if_cond(env.clone())
        ),
        env::Value("eval", inbuilt::embed_eval(env.clone())),
    ]));

    let value = match expr {
        Value::List(exprs) => inbuilt::begin_internal(env, exprs.as_ref()),
        expr @ Value::Symbol(_) => inbuilt::begin_internal(env, &[expr]),
        _ => unreachable!(),
    }?;
    println!("{value}");
    Ok(())
}

fn eval<'env>(env: Env<'env>, expr: Value<'env>) -> EvalResult<'env> {
    match expr {
        Value::List(expressions) => invoke(env, &expressions),
        Value::Symbol(str) => Ok(Value::from_env(env, str)?),
        _ => Ok(expr),
    }
}

fn invoke<'env>(env: Env<'env>, exprs: &[Value<'env>]) -> EvalResult<'env> {
    match exprs {
        [] => panic!("cannot eval {exprs:#?}"),
        [procedure, args @ ..] => {
            let procedure = eval(env.clone(), procedure.clone())?;
            match procedure {
                Value::Procedure(proc_env, proc, _) => {
                    proc(env.bind(proc_env), args)
                }
                _ => panic!("cannot call non-procedure: {procedure}"),
            }
        }
    }
}
