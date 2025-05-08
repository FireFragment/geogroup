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
