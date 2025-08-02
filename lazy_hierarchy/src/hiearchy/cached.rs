use std::hash::Hash;
use std::marker::PhantomData;
use std::{cell::RefCell, collections::HashMap};

use super::*;

/// `FGroupId` and `FLeafId` should be cheap to [clone](Clone::clone)
pub struct HierarchyCache<
    OrigGr: GroupRef,
    GroupKey: Hash,
    LeafKey: Hash,
    FGroupId: Fn(&OrigGr) -> Option<GroupKey>,
    FLeafId: Fn(&OrigGr::LeafData) -> Option<LeafKey>,
> {
    /// If returns [None], it means that this group can't be cached.
    get_group_id: FGroupId,
    /// If returns [None], it means that this leaf can't be cached.
    get_leaf_id: FLeafId,
    cache_group_data: RefCell<HashMap<GroupKey, OrigGr::GroupData>>,
    cache_group_structure: RefCell<HashMap<GroupKey, CachedGroup<GroupKey, OrigGr>>>,
    cache_leaf_data: RefCell<HashMap<LeafKey, OrigGr::LeafData>>,
}

struct CachedGroup<GroupKey, OrigGr: GroupRef> {
    id: GroupKey,
    orig_group: OrigGr,
    children: Option<Vec<CachedGroup<GroupKey, OrigGr>>>,
}

pub struct CGroupRef<
    'cache,
    OrigGr: GroupRef,
    GroupKey: Hash,
    LeafKey: Hash,
    FGroupId: Fn(&OrigGr) -> Option<GroupKey>,
    FLeafId: Fn(&OrigGr::LeafData) -> Option<LeafKey>,
> {
    this: CGroupRefBody<'cache, GroupKey, OrigGr>,
    cache: &'cache HierarchyCache<OrigGr, GroupKey, LeafKey, FGroupId, FLeafId>,
}

enum CGroupRefBody<'cache, GroupKey, OrigGr: GroupRef> {
    Cached(&'cache CachedGroup<GroupKey, OrigGr>),
    /// Used in the rare case when writing into the cache's [RefCell] failed, possibly because of attempting multiple writes simultaneously
    Uncached(OrigGr),
}

pub struct CLeafRef<
    'cache,
    OrigGr: GroupRef,
    GroupKey: Hash,
    LeafKey: Hash,
    FGroupId: Fn(&OrigGr) -> Option<GroupKey>,
    FLeafId: Fn(&OrigGr::LeafData) -> Option<LeafKey>,
> {
    this: OrigGr::LeafRef,
    cache: &'cache HierarchyCache<OrigGr, GroupKey, LeafKey, FGroupId, FLeafId>,
}

impl<
        'cache,
        OrigGr: GroupRef,
        GroupKey: Hash,
        LeafKey: Hash,
        FGroupId: Fn(&OrigGr) -> Option<GroupKey>,
        FLeafId: Fn(&OrigGr::LeafData) -> Option<LeafKey>,
    > GroupRef for CGroupRef<'cache, OrigGr, GroupKey, LeafKey, FGroupId, FLeafId>
{
    type NodeData = OrigGr::NodeData;
    type LeafData = OrigGr::LeafData;
    type GroupData = OrigGr::GroupData;
    type StructureErr = OrigGr::StructureErr;
    type LeafRef = CLeafRef<'cache, OrigGr, GroupKey, LeafKey, FGroupId, FLeafId>;

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

impl<
        'cache,
        OrigGr: GroupRef,
        GroupKey: Hash,
        LeafKey: Hash,
        FGroupId: Fn(&OrigGr) -> Option<GroupKey>,
        FLeafId: Fn(&OrigGr::LeafData) -> Option<LeafKey>,
    > LeafRef for CLeafRef<'cache, OrigGr, GroupKey, LeafKey, FGroupId, FLeafId>
{
    type LeafData = <OrigGr::LeafRef as LeafRef>::LeafData;
    type NodeData = <OrigGr::LeafRef as LeafRef>::NodeData;

    fn leaf_data(&self) -> Self::LeafData {
        todo!()
    }

    fn node_data(&self) -> Self::NodeData {
        todo!()
    }
}

impl<'cache, GroupKey, OrigGr: GroupRef> Clone for CGroupRefBody<'cache, GroupKey, OrigGr> {
    fn clone(&self) -> Self {
        match self {
            CGroupRefBody::Cached(cached_group) => CGroupRefBody::Cached(cached_group),
            CGroupRefBody::Uncached(group_ref) => CGroupRefBody::Uncached(group_ref.clone()),
        }
    }
}

impl<
        'cache,
        OrigGr: GroupRef,
        GroupKey: Hash,
        LeafKey: Hash,
        FGroupId: Fn(&OrigGr) -> Option<GroupKey>,
        FLeafId: Fn(&OrigGr::LeafData) -> Option<LeafKey>,
    > Clone for CGroupRef<'cache, OrigGr, GroupKey, LeafKey, FGroupId, FLeafId>
{
    fn clone(&self) -> Self {
        Self {
            this: self.this.clone(),
            cache: self.cache,
        }
    }
}

impl<
        'cache,
        OrigGr: GroupRef,
        GroupKey: Hash,
        LeafKey: Hash,
        FGroupId: Fn(&OrigGr) -> Option<GroupKey>,
        FLeafId: Fn(&OrigGr::LeafData) -> Option<LeafKey>,
    > Clone for CLeafRef<'cache, OrigGr, GroupKey, LeafKey, FGroupId, FLeafId>
{
    fn clone(&self) -> Self {
        Self {
            this: self.this.clone(),
            cache: self.cache.clone(),
        }
    }
}
