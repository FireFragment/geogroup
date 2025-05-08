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
    type LeafMetadata = SortItem<P, T, &'a D>;
    type NodeMetadata = ();
    type StructureErr = Infallible;
    type DoesLoading = Infallible;

    fn root(&self) -> Self::GroupRef {
        GroupRef::Root
    }

    fn get_children(
        &self,
        _group: Self::GroupRef,
    ) -> hiearchy::lazy::LoadingResult<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    {
    }

    fn node_metadata(&self, _node: hiearchy::lazy::NodeRef<Self>) -> Self::NodeMetadata {}

    fn group_metadata(&self, _group: Self::GroupRef) -> Self::GroupMetadata {}

    fn leaf_metadata(&self, leaf: Self::LeafRef) -> Self::LeafMetadata {
        self.points[leaf]
    }
}

impl<P: Point + Clone, T: Ord + Clone, D> Sorter<P, T, D> {
    pub fn new(mut points: Vec<SortItem<P, T, D>>, params: Params) -> Self {
        points.sort_unstable_by_key(|it| it.time.to_owned());
        Self { points, params }
    }
}
