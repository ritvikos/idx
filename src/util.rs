extern crate num_traits;

use std::ops::{AddAssign, Deref, SubAssign};

use num_traits::Unsigned;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Counter<T>(T)
where
    T: Copy + Unsigned + AddAssign + SubAssign;

impl<T> Counter<T>
where
    T: Copy + Unsigned + AddAssign + SubAssign,
{
    #[inline]
    pub fn new(counter: T) -> Self {
        Self(counter)
    }

    #[inline]
    pub fn inner_ref(&self) -> &T {
        &self.0
    }

    #[inline]
    pub fn increment(&mut self) {
        self.increment_by(T::one());
    }

    #[inline]
    pub fn decrement(&mut self) {
        self.decrement_by(T::one());
    }

    #[inline]
    fn increment_by(&mut self, count: T) {
        self.0 += count;
    }

    #[inline]
    fn decrement_by(&mut self, count: T) {
        self.0 -= count;
    }
}

impl<T> Deref for Counter<T>
where
    T: Copy + Unsigned + AddAssign + SubAssign,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AddAssign for Counter<T>
where
    T: Copy + Unsigned + AddAssign + SubAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl<T> SubAssign for Counter<T>
where
    T: Copy + Unsigned + AddAssign + SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}
