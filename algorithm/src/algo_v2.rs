use std::convert::Infallible;

use crate::*;
pub struct Sorter<P: Point + Clone, T: Ord + Clone, D> {
    points: Vec<SortItem<P, T, D>>,
    params: Params,
}

impl<'a, P: Point + Clone, T: Ord + Clone, D: 'a> hiearchy::Lazy for Sorter<P, T, D> {
    type GroupRef = ();
    type LeafRef = usize;
    type GroupMetadata = ();
    type NodeMetadata = ();
    type StructureErr = Infallible;
    type DoesLoading = Infallible;

    fn root(&self) -> Self::GroupRef {
        GroupRef::Root
    }
}

impl<P: Point + Clone, T: Ord + Clone, D> Sorter<P, T, D> {
    pub fn new(mut points: Vec<SortItem<P, T, D>>, params: Params) -> Self {
        points.sort_unstable_by_key(|it| it.time.to_owned());
        Self { points, params }
    }
}
