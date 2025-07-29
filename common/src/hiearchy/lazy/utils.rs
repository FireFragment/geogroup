//! Utility types for working with hierarchies

use std::marker::PhantomData;

use crate::hiearchy::concrete::Group;

use super::*;
pub mod map;
pub mod with_parent;

pub trait GroupRefUtils: GroupRef {
    /// Convert to concrete hierarchy by instantiating all the items
    fn collect_to_concrete<'c>(
        &'c self,
    ) -> Result<
        hiearchy::concrete::Group<Self::GroupMetadata, Self::LeafMetadata, Self::NodeMetadata>,
        Self::StructureErr,
    >
    where
        Self: std::marker::Sized,
    {
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

    fn map_group_data<'x, GroupDataNew: 'x, F: Fn(&Self) -> GroupDataNew + 'x>(
        self,
        fun: F,
    ) -> utils::map::AsMappedGroupRef<Self, utils::map::GroupDataMapper<F>>
    where
        Self: Sized, // TODO: Is this bound really needed?
        Self: 'x,
    {
        utils::map::map_group_data(self, fun)
    }

    fn map_leaf_data<'a, LeafDataNew, F: Fn(&Self::LeafRef) -> LeafDataNew + 'a>(
        self,
        fun: F,
    ) -> utils::map::AsMappedGroupRef<Self, utils::map::LeafDataMapper<F>>
    where
        Self: Sized,
    {
        utils::map::map_leaf_data(self, fun)
    }

    fn with_parent(self) -> utils::with_parent::WithParent<Self>
    where
        Self: Sized,
    {
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
