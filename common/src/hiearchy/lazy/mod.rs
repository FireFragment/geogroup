use std::error::Error;

use crate::hiearchy;
pub use utils::GroupRefUtils;

/// A trait for types that can provide a root group reference
///
/// `'a` is a equal to or shorter than lifetime of how long is `Self` valid
pub trait AsGroupRef<'a> {
    /// The type of group reference this hierarchy provides
    type GroupRef: GroupRef;

    /// Get the root group of the hierarchy
    fn root<'s: 'a>(&'s self) -> Self::GroupRef;
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

pub trait LeafRef {
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

pub trait AsGroupRefUtils<'a>: AsGroupRef<'a> {
    /// Convert to concrete hierarchy by instantiating all the items
    fn collect_to_concrete(
        &'a self,
    ) -> Result<
        hiearchy::Concrete<
            <Self::GroupRef as GroupRef>::GroupMetadata,
            <Self::GroupRef as GroupRef>::LeafMetadata,
            <Self::GroupRef as GroupRef>::NodeMetadata,
        >,
        <Self::GroupRef as GroupRef>::StructureErr,
    >
    where
        Self: std::marker::Sized,
    {
        Ok(hiearchy::Concrete::new(self.root().collect_to_concrete()?))
    }

    fn with_parent(&'a self) -> utils::with_parent::WithParent<'a, Self>
    where
        <Self as hiearchy::lazy::AsGroupRef<'a>>::GroupRef: Clone,
        Self: Sized,
    {
        utils::with_parent::with_parent(self)
    }
}

impl<T: GroupRef> GroupRefUtils for T {}
impl<'a, T: AsGroupRef<'a>> AsGroupRefUtils<'a> for T {}

mod utils;
