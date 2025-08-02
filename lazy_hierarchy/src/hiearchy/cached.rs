use std::hash::Hash;
use std::{cell::RefCell, collections::HashMap};

use super::*;

/// `FGroupId` and `FLeafId` should be cheap to [clone](Clone::clone)
pub struct HierarchyCache<
    OrigGr: GroupRef,
    GroupKey: Hash,
    LeafKey: Hash,
    FGroupId: Fn(&OrigGr) -> GroupKey + Clone,
    FLeafId: Fn(&OrigGr::LeafData) -> LeafKey + Clone,
> {
    get_group_id: FGroupId,
    get_leaf_id: FLeafId,
    cache_group_data: HashMap<GroupKey, FGroupId>,
    cache_leaf_data: HashMap<LeafKey, FLeafId>,
}

#[derive(Clone)]
pub struct CGroupRef<'a, OrigGr: GroupRef>(&'a Group<OrigGr>);

#[derive(Clone)]
pub struct CLeafRef<'a, OrigGr: GroupRef>(&'a Leaf<OrigGr::LeafRef>);

impl<'a, OrigGr: GroupRef> GroupRef for CGroupRef<'a, OrigGr> {
    type NodeData = OrigGr::NodeData;
    type LeafData = OrigGr::LeafData;
    type GroupData = OrigGr::GroupData;
    type StructureErr = OrigGr::StructureErr;
    type LeafRef = CLeafRef<'a, OrigGr>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        todo!()
    }

    fn group_data(&self) -> Self::GroupData {
        todo!()
    }

    fn node_data(&self) -> Self::NodeData {
        todo!()
    }
}

impl<'a, OrigGr: GroupRef> LeafRef for CLeafRef<'a, OrigGr> {
    type LeafData = <OrigGr::LeafRef as LeafRef>::LeafData;
    type NodeData = <OrigGr::LeafRef as LeafRef>::NodeData;

    fn leaf_data(&self) -> Self::LeafData {
        todo!()
    }

    fn node_data(&self) -> Self::NodeData {
        todo!()
    }
}

struct Group<OrigGr: GroupRef> {
    orig_group_ref: OrigGr,
    children: RefCell<Option<Vec<Node<OrigGr>>>>,
    group_data: RefCell<Option<OrigGr::GroupData>>,
    node_data: RefCell<Option<OrigGr::NodeData>>,
}

pub struct Leaf<OrigLf: LeafRef> {
    orig_leaf_ref: OrigLf,
    leaf_data: RefCell<Option<OrigLf::LeafData>>,
    node_data: RefCell<Option<OrigLf::NodeData>>,
}

enum Node<OrigGr: GroupRef> {
    Group(Group<OrigGr>),
    Leaf(Leaf<OrigGr::LeafRef>),
}
