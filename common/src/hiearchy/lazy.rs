use crate::UnitOrNever;

use super::concrete::GroupRef;

/// A hiearchy which is lazy, ie. it's not fully stored in RAM, but rather it initializes its parts
/// only when reading them.
///
/// Conceptually similiar to [`Iterator`] which may initialize its elements only _after_ attempt to read them.
pub trait LazyHiearchy {
    /// Reference to a leaf
    type LeafRef;
    /// Reference to a group
    type GroupRef;
    /// Additional data related to a group. See also [`Self::NodeMetadata`](LazyHiearchy::NodeMetadata)
    type GroupMetadata;
    /// Additional data related to a node
    type NodeMetadata;
    /// Additional data related to a leaf
    type LeafMetadata;
    /// Error encountered while trying to construct the group structure
    type StructureErr;

    /// Unit type if `get_children` may return [`LoadingResult::Loading`] in its methods
    /// and [`Infallible`] if not.
    type DoesLoading: UnitOrNever;

    /// Get the root group of the hiearchy
    fn root(&self) -> Self::GroupRef;

    /// Returns children of a group.
    fn get_children(
        &self,
        group: Self::GroupRef,
    ) -> LoadingResult<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>;

    fn node_metadata(&self, node: NodeRef<Self>) -> Self::NodeMetadata;
    fn group_metadata(&self, group: Self::GroupRef) -> Self::GroupMetadata;
    fn leaf_metadata(&self, group: Self::LeafRef) -> Self::LeafMetadata;
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NodeRef<H: LazyHiearchy + ?Sized> {
    Group(H::GroupRef),
    Leaf(H::LeafRef),
}

impl<H: LazyHiearchy + ?Sized> Clone for NodeRef<H>
where
    H::LeafRef: Clone,
    H::GroupRef: Clone,
{
    fn clone(&self) -> Self {
        match self {
            NodeRef::Leaf(l) => NodeRef::Leaf(l.to_owned()),
            NodeRef::Group(g) => NodeRef::Group(g.to_owned()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoadingResult<T, E> {
    Ready(Result<T, E>),
    Loading { message: String },
}

impl<T, E> LoadingResult<T, E> {
    pub fn new_ok(t: T) -> Self {
        Self::Ready(Result::Ok(t))
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> LoadingResult<U, E> {
        match self {
            LoadingResult::Ready(Ok(t)) => LoadingResult::Ready(Ok(f(t))),
            LoadingResult::Ready(Err(e)) => LoadingResult::Ready(Err(e)),
            LoadingResult::Loading { message } => LoadingResult::Loading { message },
        }
    }
}

pub use utils::*;
pub mod utils {
    use super::*;

    pub struct MapGroups<
        H: LazyHiearchy,
        NewGroupMetadata,
        F: Fn(&H, H::GroupMetadata) -> NewGroupMetadata,
    > {
        original: H,
        mapping: F,
    }

    impl<H: LazyHiearchy, NewGroupMetadata, F: Fn(&H, H::GroupMetadata) -> NewGroupMetadata>
        LazyHiearchy for MapGroups<H, NewGroupMetadata, F>
    {
        type StructureErr = H::StructureErr;
        type LeafRef = H::LeafRef;
        type GroupRef = H::GroupRef;
        type GroupMetadata = NewGroupMetadata;
        type NodeMetadata = H::NodeMetadata;
        type LeafMetadata = H::LeafMetadata;
        type DoesLoading = H::DoesLoading;
        fn root(&self) -> Self::GroupRef {
            self.original.root()
        }

        fn get_children(
            &self,
            group: Self::GroupRef,
        ) -> LoadingResult<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr> {
            self.original.get_children(group).map(|it| {
                it.map(|node| match node {
                    NodeRef::Group(g) => NodeRef::Group(g),
                    NodeRef::Leaf(l) => NodeRef::Leaf(l),
                })
            })
        }

        fn node_metadata(&self, node: NodeRef<Self>) -> Self::NodeMetadata {
            self.original.node_metadata(match node {
                NodeRef::Group(g) => NodeRef::Group(g),
                NodeRef::Leaf(l) => NodeRef::Leaf(l),
            })
        }

        fn group_metadata(&self, group: Self::GroupRef) -> Self::GroupMetadata {
            let original_metadata = self.original.group_metadata(group);
            (self.mapping)(&self.original, original_metadata)
        }

        fn leaf_metadata(&self, group: Self::LeafRef) -> Self::LeafMetadata {
            self.original.leaf_metadata(group)
        }
    }
}
