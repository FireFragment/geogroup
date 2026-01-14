use std::fmt::Debug;
use std::convert::Infallible;

/// Distance between two [points](Point). Doesn't correspond 1:1 to meters, see [ONE_METER_DISTANCE]
pub type Distance = u64;

pub const ONE_METER_DISTANCE: Distance = 65536;
pub type CameraId = u32;

/// An item that can be sorted using the geogroup algorithm
///
/// A simple and ready to use implementation is [ConcreteSortableItem]
pub trait SortableItem {
    type Time: Ord + Clone;
    type Position: Point + Clone;

    type PositionErr: Debug + Clone; // TODO: Consider removing this bound. It makes it easy to expect on the result.

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
        use geo::{GeodesicDistance, Point};
        (GeodesicDistance::geodesic_distance(&Point::from(self.to_owned()), &Point::from(rhs.to_owned())) * (ONE_METER_DISTANCE as f64)) as u64
    }
}

#[cfg(feature = "geo")]
impl Point for geo::Point {
    fn distance(&self, rhs: &Self) -> Distance {
        use geo::GeodesicDistance;
        (GeodesicDistance::geodesic_distance(self, rhs) * (ONE_METER_DISTANCE as f64)) as u64
    }
}

/// [`SortableItem`] which specifies its time and point as its fields
///
/// Generic argument `D` is for arbitrary additional data, `PE` is for errors instead of positions
#[derive(Debug)]
pub struct ConcreteSortableItem<P: Point, T: Ord + Clone, D = (), PE: Clone = Infallible> {
    pub time: T,
    pub position: Result<P, PE>,
    /// Arbitrary additional data
    pub data: D,
}


impl<P: Point + Clone, T: Ord + Clone, D, PE: std::error::Error + Clone> SortableItem for ConcreteSortableItem<P, T, D, PE> {
    type Time = T;
    type Position = P;
    type PositionErr = PE;

    fn get_time(&self) -> Self::Time {
        self.time.clone()
    }

    fn get_position(&self) -> Result<P, PE> {
        self.position.clone()
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
