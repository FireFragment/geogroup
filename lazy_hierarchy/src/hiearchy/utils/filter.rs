use either::Either;

use super::*;

/// See [`dissolve_by_key`](GroupRefUtils::dissolve_by_key)
#[derive(Clone)]
pub struct FilteredGroupRef<G: GroupRef, FnFilter: Fn(&NodeRef<G>) -> bool + Clone> {
    this_group: G,
    fun_filter: FnFilter,
}

pub fn new<G: GroupRef, FnFilter: Fn(&NodeRef<G>) -> bool + Clone>(
    this: G,
    fun_filter: FnFilter,
) -> FilteredGroupRef<G, FnFilter> {
    FilteredGroupRef {
        this_group: this,
        fun_filter,
    }
}

impl<G: GroupRef, FnFilter: Fn(&NodeRef<G>) -> bool + Clone> GroupRef for FilteredGroupRef<G, FnFilter> {
    type NodeData = G::NodeData;
    type LeafData = G::LeafData;
    type GroupData = G::GroupData;
    type StructureErr = G::StructureErr;
    type LeafRef = G::LeafRef;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        self.this_group.get_children().map(|children| {
            children
                .filter(|child| (self.fun_filter)(child))
                .map(|child| match child {
                    NodeRef::Group(this_group) => NodeRef::Group(FilteredGroupRef {
                        this_group,
                        fun_filter: self.fun_filter.clone(),
                    }),
                    NodeRef::Leaf(l) => NodeRef::Leaf(l),
                })
        })
    }

    fn group_data(&self) -> Self::GroupData {
        self.this_group.group_data()
    }

    fn node_data(&self) -> Self::NodeData {
        self.this_group.node_data()
    }
}
