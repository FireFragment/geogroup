use std::ops::Deref;

use super::*;

#[derive(Clone, Debug)]
pub enum MaybeBorrowed<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> Deref for MaybeBorrowed<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeBorrowed::Borrowed(t) => t,
            MaybeBorrowed::Owned(t) => t,
        }
    }
}

impl<'a, T> From<T> for MaybeBorrowed<'a, T> {
    fn from(value: T) -> Self {
        MaybeBorrowed::Owned(value)
    }
}

impl<'a, T> From<&'a T> for MaybeBorrowed<'a, T> {
    fn from(value: &'a T) -> Self {
        MaybeBorrowed::Borrowed(value)
    }
}
