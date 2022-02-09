use crate::globals::SessionGlobals;
use crate::solidlang::context::ty::Ty;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;

#[derive(Hash)]
pub struct PoolRef<T> {
    index: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Clone for PoolRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for PoolRef<T> {}

impl Debug for PoolRef<Ty> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        SessionGlobals::with_ty_pool(|pool| pool.get(*self).fmt(f))
    }
}

pub struct Pool<T> {
    values: Vec<T>,
    current_index: usize,
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Self {
            values: vec![],
            current_index: 0,
        }
    }

    pub fn add(&mut self, t: T) -> PoolRef<T> {
        let pool_ref = PoolRef {
            index: self.current_index,
            _marker: Default::default(),
        };
        self.current_index += 1;
        self.values.push(t);
        pool_ref
    }

    pub fn get(&self, pool_ref: PoolRef<T>) -> &T {
        &self.values[pool_ref.index]
    }

    pub fn get_mut(&mut self, pool_ref: PoolRef<T>) -> &mut T {
        &mut self.values[pool_ref.index]
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.values.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.values.iter_mut()
    }
}
