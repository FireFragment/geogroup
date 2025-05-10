//! Here be dragons. This is some seriously instense Rust. For normall usage, see [`LazyHiearchyUtils`] instead
//!
//! If you thought you understand lifetimes, try reading this code.

use super::*;
pub(crate) use map::groups::*;
pub(crate) use map::nodes::*;

mod map {
    use super::*;
    pub(crate) mod groups {
        use super::*;

        pub(crate) struct MapGroups<
            H: LazyHiearchy,
            NewGroupMetadata,
            F: Fn(H::GroupMetadata<'_>) -> NewGroupMetadata,
        > {
            original: H,
            mapping: F,
        }

        impl<
                H: LazyHiearchy,
                NewGroupMetadata,
                F: Fn(H::GroupMetadata<'_>) -> NewGroupMetadata,
            > MapGroups<H, NewGroupMetadata, F>
        {
            pub(crate) fn new(original: H, mapping: F) -> Self {
                Self { original, mapping }
            }
        }

        impl<
                H: LazyHiearchy,
                NewGroupMetadata,
                F: Fn(H::GroupMetadata<'_>) -> NewGroupMetadata,
            > LazyHiearchy for MapGroups<H, NewGroupMetadata, F>
        {
            type StructureErr = H::StructureErr;
            type LeafRef<'a>
                = H::LeafRef<'a>
            where
                Self: 'a;
            type GroupRef<'a>
                = MapGroupsGroup<'a, H, NewGroupMetadata, F>
            where
                Self: 'a;
            type GroupMetadata<'a>
                = NewGroupMetadata
            where
                Self: 'a;
            type NodeMetadata<'a>
                = H::NodeMetadata<'a>
            where
                Self: 'a;
            type LeafMetadata<'a>
                = H::LeafMetadata<'a>
            where
                Self: 'a;
            type DoesLoading = H::DoesLoading;
            fn root(&self) -> Self::GroupRef<'_> {
                MapGroupsGroup(self.original.root(), &self.mapping)
            }
        }

        pub(crate) struct MapGroupsGroup<
            'a,
            H: LazyHiearchy + 'a,
            NewGroupMetadata,
            F: for<'b> Fn(H::GroupMetadata<'b>) -> NewGroupMetadata,
        >(H::GroupRef<'a>, &'a F);

        impl<
                'a,
                H: LazyHiearchy,
                NewGroupMetadata: 'a,
                F: for<'b> Fn(H::GroupMetadata<'b>) -> NewGroupMetadata,
            > GroupRef<'a> for MapGroupsGroup<'a, H, NewGroupMetadata, F>
        {
            type Hiearchy = MapGroups<H, NewGroupMetadata, F>;

            fn get_children<'b>(
                &self,
            ) -> LoadingResult<
                impl Iterator<Item = NodeRef<'b, Self::Hiearchy>>,
                <Self::Hiearchy as LazyHiearchy>::StructureErr,
                <Self::Hiearchy as LazyHiearchy>::DoesLoading,
            >
            where
                'a: 'b,
            {
                self.0.get_children().map(|children| {
                    children.map(|child| match child {
                        NodeRef::Group(g) => NodeRef::Group(MapGroupsGroup(g, self.1)),
                        NodeRef::Leaf(l) => NodeRef::Leaf(l),
                    })
                })
            }

            fn group_metadata<'b>(&self) -> <Self::Hiearchy as hiearchy::Lazy>::GroupMetadata<'b>
            where
                'a: 'b,
            {
                (self.1)(self.0.group_metadata())
            }

            fn node_metadata<'b>(&self) -> <Self::Hiearchy as hiearchy::Lazy>::NodeMetadata<'b>
            where
                'a: 'b,
            {
                self.0.node_metadata()
            }
        }
    }

    pub(crate) mod nodes {
        use super::*;

        pub(crate) struct MapNodes<
            H: LazyHiearchy,
            NewNodeMetadata,
            F: Fn(H::NodeMetadata<'_>) -> NewNodeMetadata,
        > {
            original: H,
            mapping: F,
        }

        impl<H: LazyHiearchy, NewNodeMetadata, F: Fn(H::NodeMetadata<'_>) -> NewNodeMetadata>
            MapNodes<H, NewNodeMetadata, F>
        {
            pub(crate) fn new(original: H, mapping: F) -> Self {
                Self { original, mapping }
            }
        }

        impl<H: LazyHiearchy, NewNodeMetadata, F: Fn(H::NodeMetadata<'_>) -> NewNodeMetadata>
            LazyHiearchy for MapNodes<H, NewNodeMetadata, F>
        {
            type StructureErr = H::StructureErr;
            type LeafRef<'a>
                = MapNodesLeaf<'a, H, NewNodeMetadata, F>
            where
                Self: 'a;
            type GroupRef<'a>
                = MapNodesGroup<'a, H, NewNodeMetadata, F>
            where
                Self: 'a;
            type GroupMetadata<'a>
                = H::GroupMetadata<'a>
            where
                Self: 'a;
            type NodeMetadata<'a>
                = NewNodeMetadata
            where
                Self: 'a;
            type LeafMetadata<'a>
                = H::LeafMetadata<'a>
            where
                Self: 'a;
            type DoesLoading = H::DoesLoading;
            fn root(&self) -> Self::GroupRef<'_> {
                MapNodesGroup(self.original.root(), &self.mapping)
            }
        }

        pub(crate) struct MapNodesLeaf<
            'a,
            H: LazyHiearchy + 'a,
            NewNodeMetadata,
            F: for<'b> Fn(H::NodeMetadata<'b>) -> NewNodeMetadata,
        >(H::LeafRef<'a>, &'a F);

        impl<
                'a,
                H: LazyHiearchy + 'a,
                NewNodeMetadata,
                F: for<'b> Fn(H::NodeMetadata<'b>) -> NewNodeMetadata,
            > LeafRef for MapNodesLeaf<'a, H, NewNodeMetadata, F>
        {
            type Metadata = <H::LeafRef<'a> as LeafRef>::Metadata;

            type NodeMetadata = NewNodeMetadata;

            fn leaf_metadata(&self) -> Self::Metadata {
                self.0.leaf_metadata()
            }

            fn node_metadata(&self) -> Self::NodeMetadata {
                (self.1)(self.0.node_metadata())
            }
        }

        pub(crate) struct MapNodesGroup<
            'a,
            H: LazyHiearchy + 'a,
            NewNodeMetadata,
            F: for<'b> Fn(H::NodeMetadata<'b>) -> NewNodeMetadata,
        >(H::GroupRef<'a>, &'a F);

        impl<
                'a,
                H: LazyHiearchy,
                NewNodeMetadata: 'a,
                F: for<'b> Fn(H::NodeMetadata<'b>) -> NewNodeMetadata,
            > GroupRef<'a> for MapNodesGroup<'a, H, NewNodeMetadata, F>
        {
            type Hiearchy = MapNodes<H, NewNodeMetadata, F>;

            fn get_children<'b>(
                &self,
            ) -> LoadingResult<
                impl Iterator<Item = NodeRef<'b, Self::Hiearchy>>,
                <Self::Hiearchy as LazyHiearchy>::StructureErr,
                <Self::Hiearchy as LazyHiearchy>::DoesLoading,
            >
            where
                'a: 'b,
            {
                self.0.get_children().map(|children| {
                    children.map(|child| match child {
                        NodeRef::Group(g) => NodeRef::Group(MapNodesGroup(g, self.1)),
                        NodeRef::Leaf(l) => NodeRef::Leaf(MapNodesLeaf(l, self.1)),
                    })
                })
            }

            fn group_metadata<'b>(&self) -> <Self::Hiearchy as hiearchy::Lazy>::GroupMetadata<'b>
            where
                'a: 'b,
            {
                self.0.group_metadata()
            }

            fn node_metadata<'b>(&self) -> NewNodeMetadata
            where
                'a: 'b,
            {
                (self.1)(self.0.node_metadata())
            }
        }
    }
}
