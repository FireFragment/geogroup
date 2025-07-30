use lazy_hierarchy::*;

//pub mod algo;
pub mod bintree;
pub mod deep_sorter;
pub mod geogroup;
pub use geogroup::Sorter as GeogroupSorter;
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

/// Distance between two [points](Point)
pub type Distance = u64;
pub type DepthParam = Strength;

/// Normally, depth and [strength](Strength) are in the range `[-1..1]`.
/// This type however represents depth times [`i16::MAX`] for the best precision and ease of use compared to floats.
pub const MAX_DEPTH: Strength = i16::MAX; // TODO: Convert to struct

/// Normally, "strength" is in the range `[-1..1]`.
/// This type however represents strength times [`i16::MAX`] for the best precision and ease of use compared to floats.
pub type Strength = i16; // TODO: Convert to struct

/// An item that can be sorted using the geogroup algorithm
pub trait SortableItem {
    type Time: Ord + Clone;
    type Position: Point + Clone;

    fn get_time(&self) -> Self::Time;
    fn get_position(&self) -> Self::Position;
}

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

pub trait Point {
    fn distance(&self, rhs: &Self) -> Distance;
}

impl Point for u64 {
    fn distance(&self, rhs: &Self) -> Distance {
        self.abs_diff(*rhs)
    }
}

#[cfg(feature = "geo")]
impl Point for geo::Coord {
    fn distance(&self, rhs: &Self) -> Distance {
        use geo::EuclideanDistance;
        (self.euclidean_distance(rhs) * 65536.0) as u64
    }
}

#[cfg(feature = "geo")]
impl Point for geo::Point {
    fn distance(&self, rhs: &Self) -> Distance {
        use geo::EuclideanDistance;
        (self.euclidean_distance(rhs) * 65536.0) as u64
    }
}
