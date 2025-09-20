use std::fmt::Debug;
use std::convert::Infallible;

/// Distance between two [points](Point)
pub type Distance = u64;
pub type CameraId = u32;

/// An item that can be sorted using the geogroup algorithm
///
/// A simple and ready to use implementation is [ConcreteSortableItem]
pub trait SortableItem {
    type Time: Ord + Clone;
    type Position: Point + Clone;

    type PositionErr: Debug; // TODO: Consider removing this bound. It makes it easy to expect on the result.

    fn get_time(&self) -> Self::Time;
    fn get_position(&self) -> Result<Self::Position, Self::PositionErr>;
    fn get_camera(&self) -> CameraId {
        0
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

/// [`SortableItem`] which specifies its time and point as its fields
///
/// Generic argument `D` is for arbitrary additional data
#[derive(Debug)]
pub struct ConcreteSortableItem<P: Point, T: Ord + Clone, D = ()> {
    pub time: T,
    pub position: P,
    /// Arbitrary additional data
    pub data: D,
}

impl<P: Point + Clone, T: Ord + Clone, D> SortableItem for ConcreteSortableItem<P, T, D> {
    type Time = T;
    type Position = P;
    type PositionErr = Infallible;

    fn get_time(&self) -> Self::Time {
        self.time.clone()
    }

    fn get_position(&self) -> Result<P, Infallible> {
        Ok(self.position.clone())
    }

}

impl<T: SortableItem> SortableItem for &T {
    type Position = T::Position;
    type Time = T::Time;
    type PositionErr = T::PositionErr;

    fn get_time(&self) -> Self::Time {
        (*self).get_time()
    }

    fn get_position(&self) -> Result<Self::Position, Self::PositionErr> {
        (*self).get_position()
    }
}
