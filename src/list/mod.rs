use crate::frc::{Frc, FrcPlaceholder};
use std::fmt::{Debug, Display};

#[derive(Clone, PartialEq, Eq)]
pub enum List<T> {
    Cons(Frc<(T, List<T>)>),
    Tail,
}

impl<T> List<T> {
    pub fn any(&self, f: impl Fn(&T) -> bool) -> bool {
        self.find(f).is_some()
    }
    pub fn find(&self, f: impl Fn(&T) -> bool) -> Option<&T> {
        match self {
            List::Cons(frc) => match frc.as_ref() {
                (value, _) if f(&value) => Some(value),
                (_, tail) => tail.find(f),
            },
            List::Tail => None,
        }
    }
    pub fn first<U>(&self, f: impl Fn(&T) -> Option<U>) -> Option<U> {
        match self {
            List::Cons(frc) => {
                let (e, tail) = frc.as_ref();

                match f(e) {
                    Some(result) => Some(result),
                    None => tail.first(f),
                }
            }
            List::Tail => None,
        }
    }
    pub fn clone_map<U>(&self, f: impl Fn(&T) -> U) -> List<U> {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.as_ref();
                List::Cons(Frc::cons((f(car), cdr.clone_map(f))))
            }
            List::Tail => List::Tail,
        }
    }
    pub fn try_clone_map<U, E>(&self, f: impl Fn(&T) -> Result<U, E>) -> Result<List<U>, E> {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.as_ref();
                let car = f(car)?;
                Ok(List::Cons(Frc::cons((car, cdr.try_clone_map(f)?))))
            }
            List::Tail => Ok(List::Tail),
        }
    }
    pub fn borrow_fold<U>(&self, initial: U, f: impl Fn(&T, U) -> U) -> U {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.as_ref();
                cdr.borrow_fold(f(car, initial), f)
            }
            List::Tail => initial,
        }
    }
    pub fn get_index(&self, idx: usize) -> Option<&T> {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.as_ref();
                if idx == 0 {
                    Some(car)
                } else {
                    cdr.get_index(idx - 1)
                }
            }
            List::Tail => None,
        }
    }
}

impl <T> FrcPlaceholder for List<T> {
    fn placeholder() -> Self {
        Self::Tail
    }
}

impl<T: Clone + FrcPlaceholder> List<T> {
    pub fn map(self, f: impl Fn(T) -> T) -> Self {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.decons();
                List::Cons(Frc::cons((f(car), cdr.map(f))))
            }
            List::Tail => List::Tail,
        }
    }
    pub fn fold<U>(self, initial: U, f: impl Fn(T, U) -> U) -> U {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.decons();
                cdr.fold(f(car, initial), f)
            }
            List::Tail => initial,
        }
    }
}

impl<T: Debug> List<T> {
    fn fmt_rec_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.as_ref();
                write!(f, " {:?}", car)?;
                cdr.fmt_rec_debug(f)
            }
            List::Tail => write!(f, ")"),
        }
    }
}
impl<T: Debug> Debug for List<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.as_ref();
                write!(f, "({:?}", car)?;
                cdr.fmt_rec_debug(f)
            }
            List::Tail => write!(f, "()"),
        }
    }
}

impl<T: Display> List<T> {
    fn fmt_rec_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.as_ref();
                write!(f, " {}", car)?;
                cdr.fmt_rec_display(f)
            }
            List::Tail => write!(f, ")"),
        }
    }
}
impl<T: Display> Display for List<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            List::Cons(cons) => {
                let (car, cdr) = cons.as_ref();
                write!(f, "({}", car)?;
                cdr.fmt_rec_display(f)
            }
            List::Tail => write!(f, "()"),
        }
    }
}
