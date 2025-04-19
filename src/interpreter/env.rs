use std::{marker::PhantomData, rc::Rc, usize};

pub trait Lookup<K: PartialEq, V> {
	fn lookup<'a>(&'a self, id: &K) -> Option<&'a V>;
}

pub struct Value<K: PartialEq, V>(pub K, pub V);
impl<K: PartialEq, V> Lookup<K, V> for Value<K, V> {
	fn lookup<'a>(&'a self, id: &K) -> Option<&'a V> {
		if *id == self.0 {
			Some(&self.1)
		} else {
			None
		}
	}
}

pub struct Values<K: PartialEq, V, R: AsRef<[Value<K, V>]>>(pub R, PhantomData<K>, PhantomData<V>);
impl<K: PartialEq, V, R: AsRef<[Value<K, V>]>> Values<K, V, R> {
	pub fn new(vals: R) -> Self {
		Self(vals, PhantomData, PhantomData)
	}
}
impl<K: PartialEq, V, R: AsRef<[Value<K, V>]>> Lookup<K, V> for Values<K, V, R> {
	fn lookup<'a>(&'a self, id: &K) -> Option<&'a V> {
		for Value(value_id, value) in self.0.as_ref() {
			if id == value_id {
				return Some(value);
			}
		}
		None
	}
}

impl<K: PartialEq, V, L: Lookup<K, V>, R: Lookup<K, V>> Lookup<K, V> for (L, R) {
	fn lookup<'a>(&'a self, id: &K) -> Option<&'a V> {
		match self.0.lookup(id) {
			value @ Some(_) => value,
			None => self.1.lookup(id),
		}
	}
}

impl<K: PartialEq, V> Lookup<K, V> for () {
	fn lookup<'a>(&'a self, _: &K) -> Option<&'a V> {
		None
	}
}

pub struct NameEnv<'env, V: 'env>(Rc<dyn Lookup<&'env str, V> + 'env>);
impl<'env, V: 'env> Lookup<&'env str, V> for NameEnv<'env, V> {
	fn lookup<'a>(&'a self, id: &&'env str) -> Option<&'a V> {
		self.0.lookup(id)
	}
}
impl<'env, V: 'env> NameEnv<'env, V> {
	pub fn new() -> Self {
		Self(Rc::new(()))
	}
	pub fn bind<L: Lookup<&'env str, V> + 'env>(&self, vals: L) -> Self {
		Self(Rc::new((vals, self.clone())))
	}
}
impl<'env, V> Clone for NameEnv<'env, V> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

pub struct IndexEnv<'env, V>(usize, Rc<dyn Lookup<usize, V> + 'env>);
impl<'env, V> Lookup<usize, V> for IndexEnv<'env, V> {
	fn lookup<'a>(&'a self, id: &usize) -> Option<&'a V> {
		self.1.lookup(id)
	}
}
impl<'env, V: 'env> IndexEnv<'env, V> {
	pub fn new() -> Self {
		Self(0, Rc::new(()))
	}
	pub fn bind(&self, val: V) -> Self {
		Self(self.0 + 1, Rc::new((Value(self.0, val), self.clone())))
	}
}
impl<'env, V> Clone for IndexEnv<'env, V> {
	fn clone(&self) -> Self {
		Self(self.0, self.1.clone())
	}
}
