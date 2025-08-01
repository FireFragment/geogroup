//! A hierarchy adapter which lets you "fuse" multiple [`GroupRef`]s into a single hierarchy.
//!
//! It takes a hierarchy whose [`LeafData`](GroupRef::LeafData) is [`MainLeafData`].
//!  - Leaves matching [`MainLeafData::RealLeaf`] are left as leaves
//!  - Leaves matching [`MainLeafData::Subgroup`] are changed to groups

use super::*;

#[derive(Clone)]
pub struct FGroupRef<
    BaseGroup: GroupRef<
        GroupData = Subgroup::GroupData,
        NodeData = Subgroup::NodeData,
        LeafData = MainLeafData<Subgroup::LeafData, Subgroup>,
        StructureErr = Subgroup::StructureErr,
    >,
    Subgroup: GroupRef,
>(FGroupRefInner<BaseGroup, Subgroup>);

impl<
        BaseGroup: GroupRef<
            GroupData = Subgroup::GroupData,
            NodeData = Subgroup::NodeData,
            LeafData = MainLeafData<Subgroup::LeafData, Subgroup>,
            StructureErr = Subgroup::StructureErr,
        >,
        Subgroup: GroupRef,
    > FGroupRef<BaseGroup, Subgroup>
{
    pub fn new(base_group: BaseGroup) -> Self {
        Self(FGroupRefInner::Base(base_group))
    }
}

#[derive(Clone)]
pub enum FGroupRefInner<
    BaseGroup: GroupRef<
        GroupData = Subgroup::GroupData,
        NodeData = Subgroup::NodeData,
        LeafData = MainLeafData<Subgroup::LeafData, Subgroup>,
        StructureErr = Subgroup::StructureErr,
    >,
    Subgroup: GroupRef,
> {
    Base(BaseGroup),
    Subgroup(Subgroup),
}

pub enum MainLeafData<T, SubGroup: GroupRef> {
    RealLeaf(T),
    Subgroup(SubGroup),
}

#[derive(Clone)]
pub struct FLeafRef<
    Sg: GroupRef,
    B: LeafRef<LeafData = MainLeafData<S::LeafData, Sg>, NodeData = S::NodeData>,
    S: LeafRef,
>(FLeafRefInner<Sg, B, S>);

#[derive(Clone)]
enum FLeafRefInner<
    Sg: GroupRef,
    B: LeafRef<LeafData = MainLeafData<S::LeafData, Sg>, NodeData = S::NodeData>,
    S: LeafRef,
> {
    /// The leaf comes from the base
    ///
    /// Constructing this where `B.leaf_data()` returns [`Subgroup`](MainLeafData::Subgroup) may lead to [panics](panic)
    Base(B),
    /// The leaf comes from a [subgroup](MainLeafData::Subgroup)
    Subgroup(S),
}

impl<
        Sg: GroupRef,
        B: LeafRef<LeafData = MainLeafData<S::LeafData, Sg>, NodeData = S::NodeData>,
        S: LeafRef,
    > LeafRef for FLeafRef<Sg, B, S>
{
    type LeafData = S::LeafData;

    type NodeData = S::NodeData;

    fn leaf_data(&self) -> Self::LeafData {
        match &self.0 {
            FLeafRefInner::Base(leaf) => {
                if let MainLeafData::RealLeaf(leaf) = leaf.leaf_data() {
                    leaf
                } else {
                    panic!(
                        "FLeafRefInner::Base contains a value that returns MainLeafData::Subgroup, which is illegal"
                    )
                }
            }
            FLeafRefInner::Subgroup(leaf) => leaf.leaf_data(),
        }
    }

    fn node_data(&self) -> Self::NodeData {
        match &self.0 {
            FLeafRefInner::Base(leaf) => leaf.node_data(),
            FLeafRefInner::Subgroup(leaf) => leaf.node_data(),
        }
    }
}

impl<
        BaseGroup: GroupRef<
            GroupData = Subgroup::GroupData,
            NodeData = Subgroup::NodeData,
            LeafData = MainLeafData<Subgroup::LeafData, Subgroup>,
            StructureErr = Subgroup::StructureErr,
        >,
        Subgroup: GroupRef,
    > GroupRef for FGroupRef<BaseGroup, Subgroup>
{
    type NodeData = Subgroup::NodeData;
    type LeafData = Subgroup::LeafData;
    type GroupData = Subgroup::GroupData;
    type StructureErr = Subgroup::StructureErr;
    type LeafRef = FLeafRef<Subgroup, BaseGroup::LeafRef, Subgroup::LeafRef>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        Ok(match &self.0 {
            FGroupRefInner::Base(base_group) => {
                Either::Left(base_group.get_children()?.map(|child| match child {
                    NodeRef::Group(child_group) => {
                        NodeRef::Group(FGroupRef(FGroupRefInner::Base(child_group)))
                    }
                    NodeRef::Leaf(child_leaf) => match child_leaf.leaf_data() {
                        MainLeafData::RealLeaf(_) => {
                            NodeRef::Leaf(FLeafRef(FLeafRefInner::Base(child_leaf)))
                        }
                        MainLeafData::Subgroup(subgroup) => {
                            NodeRef::Group(FGroupRef(FGroupRefInner::Subgroup(subgroup)))
                        }
                    },
                }))
            }
            FGroupRefInner::Subgroup(group) => {
                Either::Right(group.get_children()?.map(|child| match child {
                    NodeRef::Group(g) => NodeRef::Group(FGroupRef(FGroupRefInner::Subgroup(g))),
                    NodeRef::Leaf(l) => NodeRef::Leaf(FLeafRef(FLeafRefInner::Subgroup(l))),
                }))
            }
        })
    }

    fn group_data(&self) -> Self::GroupData {
        match &self.0 {
            FGroupRefInner::Base(b) => b.group_data(),
            FGroupRefInner::Subgroup(sg) => sg.group_data(),
        }
    }

    fn node_data(&self) -> Self::NodeData {
        match &self.0 {
            FGroupRefInner::Base(b) => b.node_data(),
            FGroupRefInner::Subgroup(sg) => sg.node_data(),
        }
    }
}
