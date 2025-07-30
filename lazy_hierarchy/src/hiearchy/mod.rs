pub mod concrete;
pub use concrete::ConcreteHiearchy as Concrete;

#[cfg(test)]
mod tests;

use std::error::Error;

use crate::utils::WithParentNodeMetadata;
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
    type NodeMetadata;

    /// Additional data related to a leaf. See also [`Self::NodeMetadata`](GroupRef::NodeMetadata)
    type LeafMetadata;

    /// Additional data related to a group. See also [`Self::NodeMetadata`](GroupRef::NodeMetadata)
    type GroupMetadata;

    /// Error encountered while trying to construct the group structure
    type StructureErr: Error;

    /// The type of leaf reference
    type LeafRef: LeafRef<Metadata = Self::LeafMetadata, NodeMetadata = Self::NodeMetadata>;

    /// Returns children of a group.
    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized;

    /// Return value can't include references to self (to [`GroupRef`]),
    /// but can include references to the hierarchy this group belongs to.
    fn group_metadata(&self) -> Self::GroupMetadata;

    /// Return value can't include references to self (to [`GroupRef`]),
    /// but can include references to the hierarchy this group belongs to.
    fn node_metadata(&self) -> Self::NodeMetadata;
}

pub trait LeafRef: Clone {
    /// Additional data related to a leaf
    type Metadata;
    /// Additional data related to every node in the hierarchy
    type NodeMetadata;

    fn leaf_metadata(&self) -> Self::Metadata;
    fn node_metadata(&self) -> Self::NodeMetadata;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeRef<G: GroupRef> {
    Group(G),
    Leaf(G::LeafRef),
}

impl<G: GroupRef> NodeRef<G> {
    pub fn node_data(&self) -> G::NodeMetadata {
        match self {
            NodeRef::Group(group) => group.node_metadata(),
            NodeRef::Leaf(leaf) => leaf.node_metadata(),
        }
    }
}

impl<T: GroupRef> GroupRefUtils for T {}
impl<T: GroupRef<NodeMetadata = WithParentNodeMetadata<T>>> WithParentUtils<T> for T {}
impl<T: AsGroupRef> AsGroupRefUtils for T {}
