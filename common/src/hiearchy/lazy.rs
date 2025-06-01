use std::convert::Infallible;

use crate::hiearchy;
use crate::UnitOrNever;

/// A hiearchy which is lazy, ie. it's not fully stored in RAM, but rather it initializes its parts
/// only when reading them.
///
/// Conceptually similiar to [`Iterator`] which may initialize its elements only _after_ attempt to read them.
pub trait LazyHiearchy {
    /// Holds reference to a leaf
    type LeafRef<'a>: LeafRef<
        Metadata = Self::LeafMetadata<'a>,
        NodeMetadata = Self::NodeMetadata<'a>,
    >
    where
        Self: 'a;
    /// Holds reference to a group
    type GroupRef<'a>: GroupRef<'a, Hiearchy = Self>
    where
        Self: 'a;
    /// Additional data related to a node
    type NodeMetadata<'a>
    where
        Self: 'a;

    /// Additional data related to a leaf
    type LeafMetadata<'a>
    where
        Self: 'a;
    /// Error encountered while trying to construct the group structure
    type StructureErr;

    /// Additional data related to a group. See also [`Self::NodeMetadata`](LazyHiearchy::NodeMetadata)
    type GroupMetadata<'a>
    where
        Self: 'a;

    /// Unit type if [`get_children`](GroupRef::get_children) may return [`LoadingResult::Loading`] in its methods
    /// and [`Infallible`] if not.
    type DoesLoading: UnitOrNever;

    /// Get the root group of the hiearchy
    fn root(&self) -> Self::GroupRef<'_>;
}

/// The lifetime parameter 'a is lifetime of the [hiearchy this group belongs to](`GroupRef::Hiearchy`).
pub trait GroupRef<'a> {
    /// [Hiearchy](`LazyHiearchy`) this group belongs to
    type Hiearchy: LazyHiearchy + 'a;

    /// Returns children of a group.
    fn get_children<'b>(
        &self,
    ) -> LoadingResult<
        impl Iterator<Item = NodeRef<'b, Self::Hiearchy>>,
        <Self::Hiearchy as LazyHiearchy>::StructureErr,
        <Self::Hiearchy as LazyHiearchy>::DoesLoading,
    >
    where
        'a: 'b;

    /// Return value can't include references to self (to [`GroupRef`]),
    /// but can include references to the [hiearchy this group belongs to](`GroupRef::Hiearchy`)
    /// represented by lifetime `'a`.
    fn group_metadata<'b>(&self) -> <Self::Hiearchy as LazyHiearchy>::GroupMetadata<'b>
    where
        'a: 'b;

    /// Return value can't include references to self (to [`GroupRef`]),
    /// but can include references to the [hiearchy this group belongs to](`GroupRef::Hiearchy`)
    /// represented by lifetime `'a`.
    fn node_metadata<'b>(&self) -> <Self::Hiearchy as LazyHiearchy>::NodeMetadata<'b>
    where
        'a: 'b;
}

pub trait LeafRef {
    /// Additional data related to a leaf
    type Metadata;
    /// Additional data related to every node in the [hiearchy](LazyHiearchy)
    type NodeMetadata;

    fn leaf_metadata(&self) -> Self::Metadata;
    fn node_metadata(&self) -> Self::NodeMetadata;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeRef<'a, H: LazyHiearchy + ?Sized + 'a> {
    Group(H::GroupRef<'a>),
    Leaf(H::LeafRef<'a>),
}

#[derive(Debug, Clone)]
pub enum LoadingResult<T, E, DoesLoading: UnitOrNever> {
    Ready(Result<T, E>),
    Loading { message: String, guard: DoesLoading },
}

impl<T, E> LoadingResult<T, E, Infallible> {
    /// Will never panic, because if this function can be called,
    /// it has been garantueed on type level that this [`LoadingResult`] is always [ready](LoadingResult::Ready)
    pub fn to_result(self) -> Result<T, E> {
        match self {
            LoadingResult::Loading { message: _, guard } => match guard {},
            LoadingResult::Ready(r) => r,
        }
    }
}

impl<T, E, DoesLoading: UnitOrNever> LoadingResult<T, E, DoesLoading> {
    pub fn new_ok(t: T) -> Self {
        Self::Ready(Result::Ok(t))
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> LoadingResult<U, E, DoesLoading> {
        match self {
            LoadingResult::Ready(Ok(t)) => LoadingResult::Ready(Ok(f(t))),
            LoadingResult::Ready(Err(e)) => LoadingResult::Ready(Err(e)),
            LoadingResult::Loading { message, guard } => LoadingResult::Loading { message, guard },
        }
    }
}

pub trait LazyHiearchyUtils: LazyHiearchy {
    fn map_groups<NewGroupMetadata, F: Fn(Self::GroupMetadata<'_>) -> NewGroupMetadata>(
        self,
        mapping: F,
    ) -> utils::MapGroups<Self, NewGroupMetadata, F>
    where
        Self: std::marker::Sized,
    {
        utils::MapGroups::new(self, mapping)
    }

    fn map_nodes<NewNodeMetadata, F: Fn(Self::NodeMetadata<'_>) -> NewNodeMetadata>(
        self,
        mapping: F,
    ) -> utils::MapNodes<Self, NewNodeMetadata, F>
    where
        Self: std::marker::Sized,
    {
        utils::MapNodes::new(self, mapping)
    }
}

pub trait LazyGroupUtils<'a, H: LazyHiearchy<DoesLoading = Infallible>>:
    GroupRef<'a, Hiearchy = H> + Sized
{
    // 'a = lifetime of the hiearchy H
    // 'b = lifetime of the returned data, referring to H
    // 'c = lifetime of the GroupRef, can be arbitrarily short
    /// For now works only when the [hiearchy](LazyHiearchy) doesn't [do loading](LazyHiearchy::DoesLoading)
    fn collect_to_concrete<'b, 'c>(
        &'c self,
    ) -> Result<
        hiearchy::concrete::Group<
            <Self::Hiearchy as LazyHiearchy>::GroupMetadata<'b>,
            <Self::Hiearchy as LazyHiearchy>::LeafMetadata<'b>,
            <Self::Hiearchy as LazyHiearchy>::NodeMetadata<'b>,
        >,
        <Self::Hiearchy as LazyHiearchy>::StructureErr,
    >
    where
        'a: 'b,
    {
        let group_data = self.group_metadata();
        let node_data = self.node_metadata();
        let children = self.get_children().to_result()?;
        Ok(hiearchy::concrete::Group::new(
            {
                children
                    .map(|child| {
                        Ok(match child {
                            NodeRef::Group(g) => {
                                //let aa: () = g;
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

    // 'a = lifetime of the hiearchy H
    // 'b = lifetime of the returned data, referring to H
    // 'c = lifetime of the GroupRef, can be arbitrarily short
    /// For now works only when the [hiearchy](LazyHiearchy) doesn't [do loading](LazyHiearchy::DoesLoading)
    fn exp<'b, 'c>(&'c self) -> <Self::Hiearchy as LazyHiearchy>::GroupMetadata<'b>
    where
        'a: 'b,
    {
        self.group_metadata()
    }
}

pub trait NoLoadingLazyHiearchyUtils: LazyHiearchy<DoesLoading = Infallible> {
    /// Convert to concrete hiearchy by instantiating all the items
    fn collect_to_concrete<'a: 'b, 'b>(
        &'a self,
    ) -> Result<
        hiearchy::Concrete<Self::GroupMetadata<'b>, Self::LeafMetadata<'b>, Self::NodeMetadata<'b>>,
        Self::StructureErr,
    >
    where
        Self: std::marker::Sized,
    {
        Ok(hiearchy::Concrete::new(self.root().collect_to_concrete()?))
    }
}

impl<T: LazyHiearchy> LazyHiearchyUtils for T {}
impl<T: LazyHiearchy<DoesLoading = Infallible>> NoLoadingLazyHiearchyUtils for T {}
impl<'a, T: GroupRef<'a, Hiearchy = H>, H: LazyHiearchy<DoesLoading = Infallible>>
    LazyGroupUtils<'a, H> for T
{
}

mod utils;
