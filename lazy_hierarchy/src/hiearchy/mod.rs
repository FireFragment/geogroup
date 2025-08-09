pub mod concrete;
pub mod fused;
pub use concrete::ConcreteHiearchy as Concrete;
pub mod either_impl;
pub use either_impl::*;

#[cfg(test)]
mod tests;

use std::error::Error;

use crate::utils::WithParentNodeData;
use either::Either;
pub use utils::{AsGroupRefUtils, GroupRefUtils, WithParentUtils};

pub mod utils;

/// A trait for types that can provide a root group reference
///
/// `'a` is a equal to or shorter than lifetime of how long is `Self` valid
pub trait AsGroupRef {
    /// The type of group reference this hierarchy provides
    type GroupRef<'a>: GroupRef
    where
        Self: 'a;

    /// Get the root group of the hierarchy
    fn root<'s>(&'s self) -> Self::GroupRef<'s>;
}

pub trait GroupRef: Clone {
    /// Additional data related to any node alongside [`LeafMetadata`] and [`GroupMetadata`]
    type NodeData;

    /// Additional data related to a leaf. See also [`Self::NodeMetadata`](GroupRef::NodeMetadata)
    type LeafData;

    /// Additional data related to a group. See also [`Self::NodeMetadata`](GroupRef::NodeMetadata)
    type GroupData;

    /// Error encountered while trying to construct the group structure
    type StructureErr;

    /// The type of leaf reference
    type LeafRef: LeafRef<LeafData = Self::LeafData, NodeData = Self::NodeData>;

    /// Returns children of a group.
    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized;

    /// Return value can't include references to self (to [`GroupRef`]),
    /// but can include references to the hierarchy this group belongs to.
    fn group_data(&self) -> Self::GroupData;

    /// Return value can't include references to self (to [`GroupRef`]),
    /// but can include references to the hierarchy this group belongs to.
    fn node_data(&self) -> Self::NodeData;
}

pub trait LeafRef: Clone {
    /// Additional data related to a leaf
    type LeafData;
    /// Additional data related to every node in the hierarchy
    type NodeData;

    fn leaf_data(&self) -> Self::LeafData;
    fn node_data(&self) -> Self::NodeData;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeRef<G: GroupRef> {
    Group(G),
    Leaf(G::LeafRef),
}

impl<G: GroupRef> NodeRef<G> {
    pub fn node_data(&self) -> G::NodeData {
        match self {
            NodeRef::Group(group) => group.node_data(),
            NodeRef::Leaf(leaf) => leaf.node_data(),
        }
    }
}

impl<T: GroupRef> GroupRefUtils for T {}
impl<T: GroupRef<NodeData = WithParentNodeData<T>>> WithParentUtils<T> for T {}
impl<T: AsGroupRef> AsGroupRefUtils for T {}
