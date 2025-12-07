use std::{
    mem,
    rc::Rc,
    fmt::{Debug, Display},
};

#[derive(PartialEq, Eq)]
pub struct Frc<T: ?Sized> {
    inner: Rc<T>,
}

impl<T> Frc<T> {
    #[inline]
    pub fn cons(value: T) -> Self {
        Self {
            inner: Rc::new(value),
        }
    }
}

impl<T: ?Sized> AsRef<T> for Frc<T> {
    fn as_ref(&self) -> &T {
        &self.inner.as_ref()
    }
}

impl<T: Clone> Frc<T> {
    #[inline]
    pub fn decons(self) -> T {
        Rc::unwrap_or_clone(self.inner)
    }
}

pub trait FrcPlaceholder {
    fn placeholder() -> Self;
}

impl <A: FrcPlaceholder, B: FrcPlaceholder> FrcPlaceholder for (A, B) {
    fn placeholder() -> Self {
        (A::placeholder(), B::placeholder())
    }
}

impl<T: Clone + FrcPlaceholder> Frc<T> {
    #[inline]
    pub fn map(mut self, f: impl FnOnce(T) -> T) -> Self {
        let m = Rc::make_mut(&mut self.inner);
        *m = f(mem::replace(m, T::placeholder()));
        self
    }
}

impl<T: ?Sized> Clone for Frc<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Debug> Debug for Frc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl<T: Display> Display for Frc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
