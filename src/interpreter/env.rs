use core::fmt;
use std::{marker::PhantomData, rc::Rc};

pub trait Lookup<K: PartialEq, V> {
    fn lookup<'a>(&'a self, id: &K) -> Option<&'a V>;
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub struct Debug<K: PartialEq, V, L: Lookup<K, V>>(L, PhantomData<K>, PhantomData<V>);
impl<K: PartialEq, V, L: Lookup<K, V>> Debug<K, V, L> {
    pub fn new(lookup: L) -> Self {
        Self(lookup, PhantomData, PhantomData)
    }
}
impl<K: PartialEq, V, L: Lookup<K, V>> fmt::Debug for Debug<K, V, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.debug(f)
    }
}

pub struct Value<K: PartialEq + fmt::Debug, V: fmt::Debug>(pub K, pub V);
impl<K: PartialEq + fmt::Debug, V: fmt::Debug> Lookup<K, V> for Value<K, V> {
    fn lookup<'a>(&'a self, id: &K) -> Option<&'a V> {
        if *id == self.0 {
            Some(&self.1)
        } else {
            None
        }
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?} -> {:#?}", self.0, self.1)
    }
}

pub struct Values<K: PartialEq + fmt::Debug, V: fmt::Debug, R: AsRef<[Value<K, V>]>>(
    pub R,
    PhantomData<K>,
    PhantomData<V>,
);
impl<K: PartialEq + fmt::Debug, V: fmt::Debug, R: AsRef<[Value<K, V>]>> Values<K, V, R> {
    pub fn new(vals: R) -> Self {
        Self(vals, PhantomData, PhantomData)
    }
}
impl<K: PartialEq + fmt::Debug, V: fmt::Debug, R: AsRef<[Value<K, V>]>> Lookup<K, V>
    for Values<K, V, R>
{
    fn lookup<'a>(&'a self, id: &K) -> Option<&'a V> {
        for Value(value_id, value) in self.0.as_ref() {
            if id == value_id {
                return Some(value);
            }
        }
        None
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let last = self.0.as_ref().len() - 1;
        self.0.as_ref().iter().enumerate().try_for_each(|(i, v)| {
            v.debug(f)?;
            if last == i {
                Ok(())
            } else {
                write!(f, "\n")
            }
        })
    }
}

impl<K: PartialEq + fmt::Debug, V: fmt::Debug, L: Lookup<K, V>, R: Lookup<K, V>> Lookup<K, V>
    for (L, R)
{
    fn lookup<'a>(&'a self, id: &K) -> Option<&'a V> {
        match self.0.lookup(id) {
            value @ Some(_) => value,
            None => self.1.lookup(id),
        }
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.debug(f)?;
        write!(f, "\n")?;
        self.1.debug(f)
    }
}

impl<K: PartialEq, V: fmt::Debug> Lookup<K, V> for () {
    fn lookup<'a>(&'a self, _: &K) -> Option<&'a V> {
        None
    }

    fn debug(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

pub struct NameEnv<'env, V: 'env + fmt::Debug>(Rc<dyn Lookup<&'env str, V> + 'env>);
impl<'env, V: 'env + fmt::Debug> Lookup<&'env str, V> for NameEnv<'env, V> {
    fn lookup<'a>(&'a self, id: &&'env str) -> Option<&'a V> {
        self.0.lookup(id)
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.debug(f)
    }
}
impl<'env, V: 'env + fmt::Debug> NameEnv<'env, V> {
    pub fn new() -> Self {
        Self(Rc::new(()))
    }
    pub fn bind<L: Lookup<&'env str, V> + 'env>(&self, vals: L) -> Self {
        Self(Rc::new((vals, self.clone())))
    }
}
impl<'env, V: fmt::Debug> Clone for NameEnv<'env, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
