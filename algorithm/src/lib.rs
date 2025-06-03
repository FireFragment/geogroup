use geogroup_common::*;

//pub mod algo;
pub mod deep_sorter;
pub use algo::sort;
pub use algo::sort_just_points;

// Changing theese two may result in overflows!
// Eg. increasing capacity of Depth is dangerous, because Depth::MAX is used in the program

/// Distance between two [points](Point)
pub type Distance = u64;
pub type DepthParam = u8;

/// One item as an input to the [`sort_unordered`] function
pub struct SortItem<P: Point + Clone, T: Ord + Clone, D> {
    /// Position of the item. Should be fast to [clone](Clone).
    pub point: P,
    /// Timestamp of the item. Should be fast to [clone](Clone).
    pub time: T,
    /// Additional data, the algorithm ignores theese
    pub data: D,
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

pub enum BinTree<Leaf, InnerNode> {
    InnerNode(BTInnerNode<Leaf, InnerNode>),
    Leaf(Leaf),
}

/*impl<Leaf, InnerNode> BinTree<Leaf, InnerNode> {
    pub type InnerNode = BTInnerNode<Leaf, InnerNode>;
}*/

pub struct BTInnerNode<Leaf, InnerNode> {
    pub children: Box<[BinTree<Leaf, InnerNode>; 2]>,
    pub data: InnerNode,
}
