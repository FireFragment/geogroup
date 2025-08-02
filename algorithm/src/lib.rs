use geogroup_common::*;
use lazy_hierarchy::*;

//pub mod algo;
pub mod bintree;
pub mod deep_sorter;
pub mod geogroup;
pub use geogroup::Sorter;

#[cfg(test)]
mod tests;

pub use bintree::*;
pub use deep_sorter::{DeepSorter, StrengthInfo};
use lazy_hierarchy::prelude::*;
use std::convert::Infallible;
//pub use algo::sort;
//pub use algo::sort_just_points;

// Changing theese two may result in overflows!
// Eg. increasing capacity of Depth is dangerous, because Depth::MAX is used in the program

pub type DepthParam = Strength;

/// Normally, depth and [strength](Strength) are in the range `[-1..1]`.
/// This type however represents depth times [`i16::MAX`] for the best precision and ease of use compared to floats.
pub const MAX_DEPTH: Strength = i16::MAX; // TODO: Convert to struct

/// Normally, "strength" is in the range `[-1..1]`.
/// This type however represents strength times [`i16::MAX`] for the best precision and ease of use compared to floats.
pub type Strength = i16; // TODO: Convert to struct

/// Parameters of the algorithm influencing how it sorts
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Params {
    pub depth: DepthParam,
}

impl Default for Params {
    /// The setting of parameters I personally found reasonable (there's nothing deep in theese numbers)
    ///
    /// Value: `Params { depth: 64 }`
    fn default() -> Self {
        Params { depth: 64 }
    }
}
