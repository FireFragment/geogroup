pub use geogroup_common::*;
use lazy_hierarchy::*;

//pub mod algo;
pub mod bintree;
pub mod deep_sorter;
pub mod geogroup;
pub use geogroup::Sorter;
pub mod maybe_borrowed;
pub use maybe_borrowed::*;
pub mod type_level_logic;
pub use type_level_logic::True as TyTrue;
pub use type_level_logic::False as TyFalse;
pub use type_level_logic::TyBool;
pub use type_level_logic::TyOption;
pub use geogroup::NameStatus;

#[cfg(test)]
mod tests;

pub use bintree::*;
pub use deep_sorter::{DeepSorter, StrengthInfo};
use lazy_hierarchy::prelude::*;
use std::convert::Infallible;
use either::Either;
//pub use algo::sort;
//pub use algo::sort_just_points;

// Changing theese two may result in overflows!
// Eg. increasing capacity of Depth is dangerous, because Depth::MAX is used in the program


/// Normally, depth and [strength](Strength) are in the range `[-1..1]`.
/// This type however represents depth times [`i16::MAX`] for the best precision and ease of use compared to floats.
pub type DepthParam = Strength;

/// The maximum value that should be assigned to [`DepthParam`] (corresponding to user-facing value of depth `1`)
pub const MAX_DEPTH: Strength = i16::MAX; // TODO: Convert to struct

/// Normally, "strength" is in the range `[-1..1]`.
/// This type however represents strength times [`i16::MAX`] for the best precision and ease of use compared to floats.
pub type Strength = i16; // TODO: Convert to struct

/// Parameters of the algorithm influencing how it sorts
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Params {
    pub depth: DepthParam,
    /// The minimum distance of items that are not in the same group alone.
    /// No two items going right after each other that are closer than this distance
    /// can be separated.
    pub minimum_distance: Distance,
}

impl Default for Params {
    /// The setting of parameters I personally found reasonable (there's nothing deep in these numbers)
    fn default() -> Self {
        Params { depth: MAX_DEPTH / 2, minimum_distance: 100 * ONE_METER_DISTANCE }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct FirstLast<T> {
    pub first: T,
    pub last: T,
}
