extern crate num_traits;

use std::ops::{AddAssign, Deref, SubAssign};

use num_traits::Unsigned;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Counter<T>(T)
where
    T: Unsigned + AddAssign + SubAssign;

impl<T> Counter<T>
where
    T: Unsigned + AddAssign + SubAssign,
{
    pub fn new(counter: T) -> Self {
        Self(counter)
    }

    pub fn increment(&mut self) {
        self.increment_by(T::one());
    }

    pub fn decrement(&mut self) {
        self.decrement_by(T::one());
    }

    fn increment_by(&mut self, count: T) {
        self.0 += count;
    }

    fn decrement_by(&mut self, count: T) {
        self.0 -= count;
    }
}

impl<T> Deref for Counter<T>
where
    T: Unsigned + AddAssign + SubAssign,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
