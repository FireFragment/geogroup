/// Distance between two [points](Point)
pub type Distance = u64;

/// An item that can be sorted using the geogroup algorithm
///
/// A simple and ready to use implementation is [ConcreteSortableItem]
pub trait SortableItem {
    type Time: Ord + Clone;
    type Position: Point + Clone;

    fn get_time(&self) -> Self::Time;
    // TODO: Allow for failures
    fn get_position(&self) -> Self::Position;
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

    fn get_time(&self) -> Self::Time {
        self.time.clone()
    }

    fn get_position(&self) -> Self::Position {
        self.position.clone()
    }
}

impl<T: SortableItem> SortableItem for &T {
    type Time = T::Time;

    type Position = T::Position;

    fn get_time(&self) -> Self::Time {
        (*self).get_time()
    }

    fn get_position(&self) -> Self::Position {
        (*self).get_position()
    }
}
