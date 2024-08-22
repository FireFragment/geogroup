use geogroup_common::*;

pub mod algo;
pub use algo::sort;

// Changing theese two may result in overflows!
// Eg. increasing capacity of Depth is dangerous, because Depth::MAX is used in the program

/// Distance between two [points](Point)
pub type Distance = u64;
pub type Depth = u8;

pub trait Point {
    fn distance(&self, rhs: &Self) -> Distance;
}

pub struct Config {
    pub flatness: u8,
}

impl Point for u64 {
    fn distance(&self, rhs: &Self) -> Distance {
        self.abs_diff(*rhs)
    }
}

pub enum BinTree<Leaf, InnerNode> {
    InnerNode {
        children: Box<[BinTree<Leaf, InnerNode>; 2]>,
        data: InnerNode,
    },
    Leaf(Leaf),
}

impl<T> From<BinTree<T, ()>> for HiearchyItem<T> {
    fn from(value: BinTree<T, ()>) -> Self {
        match value {
            BinTree::InnerNode { children, data: _ } => {
                HiearchyItem::Group(children.map(From::from).into())
            }
            BinTree::Leaf(data) => HiearchyItem::Item(data),
        }
    }
}
