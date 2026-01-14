use std::hash::Hash;
use std::{convert::Infallible, fmt::Debug, ops::Deref};

pub type False = Infallible;
pub type True = ();

pub trait TyBool: Clone + Copy + Hash + Eq + Debug {
    type Not: TyBool;
}

impl TyBool for True {
    type Not = False;
}

impl TyBool for False {
    type Not = True;
}

/// Value that is present if and only if the `Present` generic param is inhibited ([`True`])
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum TyOption<T, Present: TyBool> {
    Present(T, Present),
    Empty(Present::Not),
}

impl<T> From<T> for TyOption<T, True> {
    fn from(value: T) -> Self {
        TyOption::Present(value, True::default())
    }
}

impl<T> TyOption<T, False> {
    pub fn new_empty() -> Self {
        TyOption::Empty(True::default())
    }
}

impl<T> Default for TyOption<T, False> {
    fn default() -> Self {
        TyOption::new_empty()
    }
}

impl<T, Present: TyBool> TyOption<T, Present> {
    pub fn as_ref(&self) -> TyOption<&T, Present> {
        match self {
            TyOption::Present(t, tr) => TyOption::Present(t, *tr),
            TyOption::Empty(fa) => TyOption::Empty(*fa),
        }
    }
}

impl<T> TyOption<T, True> {
    /// Unwraps the value - the fact that this contains the value is checked at compile time, therefore, this is an infallible function.
    pub fn uwnrap_infallible(self) -> T {
        match self {
            TyOption::Present(t, _) => t,
        }
    }
}

impl<T> Deref for TyOption<T, True>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            TyOption::Present(t, _) => t,
            TyOption::Empty(f) => match *f {},
        }
    }
}

/*impl<T> From<PresentIf<T, True>> for T
{
    fn from(value: PresentIf<T, True>) -> Self {
        match value {
            PresentIf::Present(t, _) => t,
        }
    }
}*/
