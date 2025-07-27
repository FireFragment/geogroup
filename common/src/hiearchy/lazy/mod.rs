use crate::hiearchy;
use utils::GroupRefUtils;

/// A trait for types that can provide a root group reference
///
/// `'a` is a equal to or shorter than lifetime of how long is `Self` valid
pub trait AsGroupRef<'a> {
    /// The type of group reference this hierarchy provides
    type GroupRef: GroupRef<'a>;

    /// Get the root group of the hierarchy
    fn root<'s: 'a>(&'s self) -> Self::GroupRef;
}

/// The lifetime parameter 'a is lifetime of the hierarchy this group belongs to.
pub trait GroupRef<'a> {
    /// Additional data related to any node alongside [`LeafMetadata`] and [`GroupMetadata`]
    type NodeMetadata;

    /// Additional data related to a leaf. See also [`Self::NodeMetadata`](GroupRef::NodeMetadata)
    type LeafMetadata;

    /// Additional data related to a group. See also [`Self::NodeMetadata`](GroupRef::NodeMetadata)
    type GroupMetadata;

    /// Error encountered while trying to construct the group structure
    type StructureErr;

    /// The type of leaf reference
    type LeafRef: LeafRef<Metadata = Self::LeafMetadata, NodeMetadata = Self::NodeMetadata>;

    /// Returns children of a group.
    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<'a, Self>>, Self::StructureErr>
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

/// The lifetime parameter 'a is lifetime of the hierarchy this node belongs to
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeRef<'a, G: GroupRef<'a>> {
    Group(G),
    Leaf(G::LeafRef),
}

pub trait AsGroupRefUtils<'a>: AsGroupRef<'a> {
    /// Convert to concrete hierarchy by instantiating all the items
    fn collect_to_concrete(
        &'a self,
    ) -> Result<
        hiearchy::Concrete<
            <Self::GroupRef as GroupRef<'a>>::GroupMetadata,
            <Self::GroupRef as GroupRef<'a>>::LeafMetadata,
            <Self::GroupRef as GroupRef<'a>>::NodeMetadata,
        >,
        <Self::GroupRef as GroupRef<'a>>::StructureErr,
    >
    where
        Self: std::marker::Sized,
    {
        Ok(hiearchy::Concrete::new(self.root().collect_to_concrete()?))
    }

    fn map_group_data<
        GroupDataNew: 'a,
        F: Fn(<Self::GroupRef as GroupRef<'a>>::GroupMetadata) -> GroupDataNew + 'a,
    >(
        &'a self,
        fun: F,
    ) -> utils::map::AsMappedGroupRef<
        'a,
        GroupDataNew,
        <Self::GroupRef as GroupRef<'a>>::LeafMetadata,
        <Self::GroupRef as GroupRef<'a>>::NodeMetadata,
        Self,
        impl Fn(
                <<Self as AsGroupRef<'a>>::GroupRef as GroupRef<'a>>::GroupMetadata,
                <Self::GroupRef as GroupRef<'a>>::NodeMetadata,
            ) -> GroupDataNew
            + 'a,
        impl Fn(
                <Self::GroupRef as GroupRef<'a>>::LeafMetadata,
                <Self::GroupRef as GroupRef<'a>>::NodeMetadata,
            ) -> <Self::GroupRef as GroupRef<'a>>::LeafMetadata
            + 'static,
        impl Fn(
                <Self::GroupRef as GroupRef<'a>>::GroupMetadata,
                <Self::GroupRef as GroupRef<'a>>::NodeMetadata,
            ) -> <Self::GroupRef as GroupRef<'a>>::NodeMetadata
            + 'static,
        impl Fn(
                <Self::GroupRef as GroupRef<'a>>::LeafMetadata,
                <Self::GroupRef as GroupRef<'a>>::NodeMetadata,
            ) -> <Self::GroupRef as GroupRef<'a>>::NodeMetadata
            + 'static,
    >
    where
        Self: Sized, // TODO: Is this bound really needed?
    {
        utils::map::map_group_data(self, fun)
    }
}

impl<'a, T: GroupRef<'a>> GroupRefUtils<'a> for T {}
impl<'a, T: AsGroupRef<'a>> AsGroupRefUtils<'a> for T {}

mod utils;
