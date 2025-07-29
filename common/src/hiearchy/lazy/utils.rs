//! Utility types for working with hierarchies

use std::marker::PhantomData;

use crate::hiearchy::concrete::Group;

use super::*;
pub mod map;
pub mod with_parent;
pub use with_parent::{WithParentGroupRef, WithParentLeafRef, WithParentNodeMetadata};

pub trait GroupRefUtils: GroupRef {
    /// Convert to concrete hierarchy by instantiating all the items
    fn collect_to_concrete<'c>(
        &'c self,
    ) -> Result<
        hiearchy::concrete::Group<Self::GroupMetadata, Self::LeafMetadata, Self::NodeMetadata>,
        Self::StructureErr,
    > {
        let group_data = self.group_metadata();
        let node_data = self.node_metadata();
        let children = self.get_children()?;
        Ok(hiearchy::concrete::Group::new(
            {
                children
                    .map(|child| {
                        Ok(match child {
                            NodeRef::Group(g) => {
                                hiearchy::concrete::Node::new_group(g.collect_to_concrete()?)
                            }
                            NodeRef::Leaf(l) => hiearchy::concrete::Node::new_leaf(
                                hiearchy::concrete::Leaf::new(l.leaf_metadata(), l.node_metadata()),
                            ),
                        })
                    })
                    .collect::<Result<_, _>>()?
            },
            group_data,
            node_data,
        ))
    }

    fn map<M: utils::map::Mapper<Self>>(self, mapper: M) -> utils::map::MappedGroupRef<Self, M> {
        utils::map::map(self, mapper)
    }

    fn map_as_groupref<M: utils::map::Mapper<Self>>(
        self,
        mapper: M,
    ) -> utils::map::MappedGroupRef<Self, M> {
        utils::map::map_as_groupref(self, mapper)
    }

    fn map_group_data<GroupDataNew, F: Fn(&Self) -> GroupDataNew + Clone>(
        self,
        fun: F,
    ) -> impl GroupRef<
        GroupMetadata = GroupDataNew,
        LeafMetadata = Self::LeafMetadata,
        NodeMetadata = Self::NodeMetadata,
        StructureErr = Self::StructureErr,
    > {
        utils::map::map_group_data(self, fun)
    }

    fn map_leaf_data<
        'a,
        LeafDataNew,
        F: Fn(&Self::LeafRef) -> LeafDataNew + 'a + std::clone::Clone,
    >(
        self,
        fun: F,
    ) -> impl GroupRef<
        GroupMetadata = Self::GroupMetadata,
        LeafMetadata = LeafDataNew,
        NodeMetadata = Self::NodeMetadata,
        StructureErr = Self::StructureErr,
    > {
        utils::map::map_leaf_data(self, fun)
    }

    fn map_node_data<
        'a,
        NodeDataNew,
        F: Fn(NodeRef<Self>) -> NodeDataNew + 'a + std::clone::Clone,
    >(
        self,
        fun: F,
    ) -> impl GroupRef<
        GroupMetadata = Self::GroupMetadata,
        LeafMetadata = Self::LeafMetadata,
        NodeMetadata = NodeDataNew,
        StructureErr = Self::StructureErr,
    > {
        utils::map::map_node_data(self, fun)
    }

    fn with_parent(self) -> utils::with_parent::WithParentGroupRef<Self> {
        utils::with_parent::with_parent(self)
    }

    /* TODO: WTF is this?
    /// For now works only when the hierarchy doesn't do loading
    fn exp<'b, 'c>(&'c self) -> Self::GroupMetadata<'b>
    where
        'a: 'b,
    {
        self.group_metadata()
    }*/
}

pub trait WithParentUtils<G: GroupRef>: GroupRef<NodeMetadata = WithParentNodeMetadata<G>> {
    /*    /// Reverse the [`with_parent`] operation.
    ///
    /// Common pattern is to call [`with_parent`], then map using the parent and finally get
    /// rid of the parent using [`without_parent`].
    fn without_parent(
        self,
    ) -> impl GroupRef<
        GroupMetadata = Self::GroupMetadata,
        LeafMetadata = Self::LeafMetadata,
        NodeMetadata = G::NodeMetadata,
        StructureErr = Self::StructureErr,
    > {
        self.map_node_data(|node_ref| node_ref.node_data().data)
    }*/
}

pub trait AsGroupRefUtils: AsGroupRef {
    /// Convert to concrete hierarchy by instantiating all the items
    fn collect_to_concrete<'a>(
        &'a self,
    ) -> Result<
        hiearchy::Concrete<
            <Self::GroupRef<'a> as GroupRef>::GroupMetadata,
            <Self::GroupRef<'a> as GroupRef>::LeafMetadata,
            <Self::GroupRef<'a> as GroupRef>::NodeMetadata,
        >,
        <Self::GroupRef<'a> as GroupRef>::StructureErr,
    >
    where
        Self: std::marker::Sized,
    {
        Ok(hiearchy::Concrete::new(self.root().collect_to_concrete()?))
    }
}
