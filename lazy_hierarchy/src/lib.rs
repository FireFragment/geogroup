use std::convert::Infallible;

pub mod hiearchy;

pub trait UnitOrNever: sealed::Sealed {}

impl UnitOrNever for () {}
impl UnitOrNever for Infallible {}

mod sealed {
    use std::convert::Infallible;

    pub(super) trait Sealed {}

    impl Sealed for () {}
    impl Sealed for Infallible {}
}

/// Doesn't actually `use` anything named, just to make trait methods available.
pub mod prelude {
    pub use super::hiearchy::lazy::{
        AsGroupRef as _, GroupRef as _, GroupRefUtils as _, LeafRef as _, WithParentUtils as _,
    };
}
