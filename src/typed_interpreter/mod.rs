use std::{cmp::max, rc::Rc};

use crate::{
    frc::{Frc, FrcPlaceholder},
    list::List,
};

#[derive(Clone)]
struct Env<T: Clone + 'static>(Rc<dyn Fn(&str) -> Option<T>>);
impl<T: Clone + 'static> Env<T> {
    fn cons() -> Self {
        Env(Rc::new(|_| None))
    }
    fn lookup(&self, sym: &str) -> Option<T> {
        self.0(sym)
    }
    fn bind_dyn(self, f: impl Fn(&str) -> Option<T> + 'static) -> Self {
        Env(Rc::new(move |sym| f(sym).or_else(|| self.lookup(sym))))
    }
    fn bind_static(self, bind_sym: &'static str, expr: T) -> Self {
        Env(Rc::new(move |sym| {
            if *sym == *bind_sym {
                Some(expr.clone())
            } else {
                self.lookup(sym)
            }
        }))
    }
    fn bind(self, bind_sym: Rc<str>, expr: T) -> Self {
        Env(Rc::new(move |sym| {
            if *sym == *bind_sym {
                Some(expr.clone())
            } else {
                self.lookup(sym)
            }
        }))
    }
}

#[derive(Clone, Debug)]
enum Object {
    Nil,
    Bool(bool),
    Sym(Rc<str>),
    Syntax(List<Object>),
    Type(Frc<Type>),
    Procedure(Procedure),
    List(List<Object>),
    Index(List<Type>, usize, Frc<Object>),
    Constructed(usize, Frc<Object>),
}

#[derive(Clone, Debug)]
enum Procedure {
    Constructed {
        ty: Frc<ProcedureType>,
        arg: Rc<str>,
        body: Frc<Object>,
    },

    Quasiquote,
    Unquote,
}

impl Procedure {
    fn type_of(&self) -> Option<Frc<ProcedureType>> {
        match self {
            Procedure::Constructed { ty, .. } => Some(ty.clone()),
            Procedure::Quasiquote => None,
            Procedure::Unquote => None,
        }
    }
    fn apply(self, ctx: Context, arg: Object) -> Result<(Context, Object), Error> {
        match self {
            Procedure::Constructed { ty, arg, body } => todo!(),
            Procedure::Quasiquote => todo!(),
            Procedure::Unquote => todo!(),
        }
    }
}

impl FrcPlaceholder for Object {
    fn placeholder() -> Self {
        Object::Nil
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Never,
    Nil,
    Bool,
    Sym,
    Type(usize),
    Procedure(Frc<ProcedureType>),
    Product(List<Type>),
    Union(List<Type>),
    Constructed(usize, Frc<Type>),
    Quote(Frc<Type>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProcedureType {
    arg: Type,
    ret: Type,
}

#[derive(Clone)]
struct ConstructedTypes {
    len: usize,
    types: List<Type>,
}

impl ConstructedTypes {
    fn get(&self, ty: usize) -> Type {
        self.types.get_index(self.len - ty).unwrap().clone()
    }
}

#[derive(Clone)]
struct Context {
    env: Env<Object>,
    ty_env: Env<Type>,
    types: ConstructedTypes,
}

impl Type {
    fn stage_of(self: &Type) -> usize {
        match self {
            Type::Never | Type::Nil | Type::Bool => 0,
            Type::Sym => 1,
            Type::Type(s) => s + 1,
            Type::Procedure(ty) => max(ty.as_ref().arg.stage_of(), ty.as_ref().ret.stage_of()),
            Type::Product(list) | Type::Union(list) => {
                list.borrow_fold(0, |ty, s| max(s, ty.stage_of()))
            }
            Type::Constructed(_, ty) => ty.as_ref().stage_of(),
            Type::Quote(ty) => ty.as_ref().stage_of() + 1,
        }
    }
}

fn type_of(ctx: &Context, obj: &Object) -> Result<Type, Error> {
    match obj {
        Object::Nil => Ok(Type::Nil),
        Object::Bool(_) => Ok(Type::Bool),
        Object::Sym(_) => Ok(Type::Sym),
        Object::Syntax(obj) => {
            type_eval(ctx.clone(), Object::Syntax(obj.clone())).map(|(_, ty)| ty)
        }
        Object::Procedure(proc) => Ok(Type::Procedure(proc.type_of())),
        Object::List(list) => Ok(Type::Product(list.try_clone_map(|obj| type_of(ctx, obj))?)),
        Object::Index(list, _, _) => Ok(Type::Union(list.clone())),
        Object::Constructed(i, _) => Ok(ctx.types.get(*i)),
        Object::Type(t) => Ok(Type::Type(t.as_ref().stage_of())),
    }
}

enum Error {
    UnboundSym(Rc<str>),
    ExpectedProcedure,
    Inapplicable(Object),
    InapplicableType(Type),
    IncorrectType { expected: Type, found: Type },
}

enum Failure {
    TypeMismatch(List<Type>),
    Err(Error),
}

fn type_eval_list(ctx: &Context, obj: List<Object>) -> Result<List<Type>, Error> {
    match obj {
        List::Cons(cons) => {
            let (car, cdr) = cons.decons();
            let (_, car) = type_eval(ctx.clone(), car)?;
            Ok(List::Cons(Frc::cons((car, type_eval_list(ctx, cdr)?))))
        }
        List::Tail => Ok(List::Tail),
    }
}

fn type_eval(ctx: Context, obj: Object) -> Result<(Context, Type), Error> {
    match obj {
        Object::Sym(sym) => match ctx.ty_env.lookup(&sym) {
            Some(ty) => Ok((ctx, ty)),
            None => Err(Error::UnboundSym(sym)),
        },
        Object::Syntax(list) => match list {
            List::Cons(cons) => {
                let (car, cdr) = cons.decons();

                let (ctx, car) = type_eval(ctx, car)?;

                let Type::Procedure(car) = car else {
                    return Err(Error::InapplicableType(car));
                };

                let car = car.decons();

                // TODO: atm cdr gets bundled into a product here, but later it might be some other
                // type that is more complex
                let cdr = Type::Product(type_eval_list(&ctx, cdr)?);

                if car.arg == cdr {
                    Ok((ctx, car.ret))
                } else {
                    Err(Error::IncorrectType {
                        expected: car.arg,
                        found: cdr,
                    })
                }
            }
            List::Tail => Err(Error::ExpectedProcedure),
        },
        _ => {
            let ty = type_of(&ctx, &obj)?;
            Ok((ctx, ty))
        }
    }
}

fn eval(ctx: Context, obj: Object) -> Result<(Context, Object), Error> {
    match obj {
        Object::Sym(sym) => match ctx.env.lookup(&sym) {
            Some(obj) => Ok((ctx, obj)),
            None => Err(Error::UnboundSym(sym)),
        },
        Object::Syntax(list) => match list {
            List::Cons(cons) => {
                let (car, cdr) = cons.decons();

                let (ctx, car) = eval(ctx, car)?;

                let Object::Procedure(proc) = car else {
                    return Err(Error::InapplicableType(type_of(&ctx, &car)?));
                };

                todo!()
            }
            List::Tail => Err(Error::ExpectedProcedure),
        },
        _ => Ok((ctx, obj)),
    }
}
