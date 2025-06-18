//! Here be dragons. This is some seriously instense Rust. For normall usage, see [`LazyHiearchyUtils`] instead
//!
//! If you thought you understand lifetimes, try reading this code.

use super::*;

pub(crate) use map::groups::*;
pub(crate) use map::nodes::*;
pub(crate) use with_parent::*;

pub use with_parent::WithParentNodeMetadata;

mod map {
    use super::*;
    pub(crate) mod groups {
        use super::*;

        pub struct MapGroups<
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

            fn reborrow_groupref<'long: 'short, 'short>(
                it: MapGroupsGroup<'long, H, NewGroupMetadata, F>,
            ) -> MapGroupsGroup<'short, H, NewGroupMetadata, F> {
                MapGroupsGroup(H::reborrow_groupref(it.0), it.1)
            }
        }

        pub struct MapGroupsGroup<
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

        pub struct MapNodes<
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

            fn reborrow_groupref<'long: 'short, 'short>(
                it: MapNodesGroup<'long, H, NewNodeMetadata, F>,
            ) -> MapNodesGroup<'short, H, NewNodeMetadata, F> {
                MapNodesGroup(H::reborrow_groupref(it.0), it.1)
            }
        }

        pub struct MapNodesLeaf<
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

        pub struct MapNodesGroup<
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

mod with_parent {
    use std::marker::PhantomData;

    use super::*;

    pub struct WithParent<H: LazyHiearchy>(H);

    impl<H: LazyHiearchy> WithParent<H> {
        pub fn new(hierarchy: H) -> Self {
            Self(hierarchy)
        }
    }

    /// `'a` is lifetime of the [underlying hiearchy](WithParent)
    pub struct WithParentNodeMetadata<'a, H: LazyHiearchy + 'a> {
        pub data: H::NodeMetadata<'a>,
        pub parent: Option<H::GroupRef<'a>>,
    }

    pub struct WithParentGroupRef<'a, H: LazyHiearchy + 'a> {
        this: H::GroupRef<'a>,
        parent: Option<H::GroupRef<'a>>,
    }

    pub struct WithParentLeafRef<'a, H: LazyHiearchy + 'a> {
        this: H::LeafRef<'a>,
        parent: Option<H::GroupRef<'a>>,
    }

    impl<'a, H: LazyHiearchy + 'a> LeafRef for WithParentLeafRef<'a, H>
    where
        for<'x> H::GroupRef<'x>: Clone,
    {
        type Metadata = H::LeafMetadata<'a>;

        type NodeMetadata = WithParentNodeMetadata<'a, H>;

        fn leaf_metadata(&self) -> Self::Metadata {
            self.this.leaf_metadata()
        }

        fn node_metadata(&self) -> Self::NodeMetadata {
            WithParentNodeMetadata {
                data: self.this.node_metadata(),
                parent: self.parent.clone(), //.map(H::reborrow_groupref),
            }
        }
    }

    impl<'a, H: LazyHiearchy> GroupRef<'a> for WithParentGroupRef<'a, H>
    where
        for<'x> H::GroupRef<'x>: Clone,
    {
        fn node_metadata<'b>(&self) -> WithParentNodeMetadata<'b, H>
        where
            'a: 'b,
        {
            WithParentNodeMetadata {
                data: self.this.node_metadata(),
                parent: self.parent.clone().map(H::reborrow_groupref),
            }
        }

        fn group_metadata<'b>(&self) -> <Self::Hiearchy as LazyHiearchy>::GroupMetadata<'b>
        where
            'a: 'b,
        {
            self.this.group_metadata()
        }

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
            self.this.get_children().map(move |it| {
                it.map(|child| match child {
                    NodeRef::Group(group) => NodeRef::Group(WithParentGroupRef {
                        this: group,
                        parent: Some(H::reborrow_groupref(self.this.clone())),
                    }),
                    NodeRef::Leaf(leaf) => NodeRef::Leaf(WithParentLeafRef {
                        this: leaf,
                        parent: Some(H::reborrow_groupref(self.this.clone())),
                    }),
                })
            })
        }

        type Hiearchy = WithParent<H>;
    }

    impl<H: LazyHiearchy> LazyHiearchy for WithParent<H>
    where
        for<'x> <H as hiearchy::lazy::LazyHiearchy>::GroupRef<'x>: std::clone::Clone,
    {
        type DoesLoading = H::DoesLoading;
        type GroupMetadata<'a>
            = H::GroupMetadata<'a>
        where
            Self: 'a;

        type LeafMetadata<'a>
            = H::LeafMetadata<'a>
        where
            Self: 'a;

        type NodeMetadata<'a>
            = WithParentNodeMetadata<'a, H>
        where
            Self: 'a;

        type StructureErr = H::StructureErr;

        type LeafRef<'a>
            = WithParentLeafRef<'a, H>
        where
            Self: 'a;

        type GroupRef<'a>
            = WithParentGroupRef<'a, H>
        where
            H: 'a;

        fn root(&self) -> Self::GroupRef<'_> {
            WithParentGroupRef {
                this: self.0.root(),
                parent: None,
            }
        }

        fn reborrow_groupref<'long: 'short, 'short>(
            it: WithParentGroupRef<'long, H>,
        ) -> WithParentGroupRef<'short, H> {
            WithParentGroupRef {
                this: H::reborrow_groupref(it.this),
                parent: it.parent.map(H::reborrow_groupref),
            }
        }
    }
}

mod flatten_by_key {
    use super::*;

    pub struct FlattenByKey<H: LazyHiearchy, F: Fn(&H::GroupRef<'_>) -> bool>(H, F);

    pub struct FlattenByKeyGroup<'a, H: LazyHiearchy + 'a, F: Fn(&H::GroupRef<'_>) -> bool + 'a> {
        this: H::GroupRef<'a>,
        map_fn: &'a F,
    }

    impl<'a, H: LazyHiearchy + 'a, F: Fn(&H::GroupRef<'_>) -> bool + 'a> GroupRef<'a>
        for FlattenByKeyGroup<'a, H, F>
    {
        type Hiearchy = FlattenByKey<H, F>;

        fn get_children<'b>(
            &self,
        ) -> LoadingResult<
            std::iter::Once<NodeRef<'b, Self::Hiearchy>>,
            //impl Iterator<Item = NodeRef<'b, Self::Hiearchy>>,
            <Self::Hiearchy as LazyHiearchy>::StructureErr,
            <Self::Hiearchy as LazyHiearchy>::DoesLoading,
        >
        where
            'a: 'b,
        {
            todo!()
            /*self.this.get_children().map(|children| {
                children.flat_map(|child| match child {
                    NodeRef::Group(group) => {
                        if (self.map_fn)(&group) {
                            Either::Left(group.get_children().map(|v| {
                                v.map(|child| match child {
                                    NodeRef::Group(_) => todo!(),
                                    NodeRef::Leaf(_) => todo!(),
                                })
                            }))
                        } else {
                            Either::Right(std::iter::once(NodeRef::Group(FlattenByKeyGroup {
                                this: group,
                                map_fn: self.map_fn, // Preserve group without flattening
                            })))
                        }
                    }
                    NodeRef::Leaf(leaf) => todo!(), //NodeRef::Leaf(leaf),
                })
            })*/
        }

        fn group_metadata<'b>(&self) -> <Self::Hiearchy as LazyHiearchy>::GroupMetadata<'b>
        where
            'a: 'b,
        {
            self.this.group_metadata()
        }

        fn node_metadata<'b>(&self) -> <Self::Hiearchy as LazyHiearchy>::NodeMetadata<'b>
        where
            'a: 'b,
        {
            self.this.node_metadata()
        }
    }

    impl<H: LazyHiearchy, F: Fn(&H::GroupRef<'_>) -> bool> LazyHiearchy for FlattenByKey<H, F> {
        type LeafRef<'x>
            = H::LeafRef<'x>
        where
            Self: 'x;
        type StructureErr = H::StructureErr;
        type NodeMetadata<'x>
            = H::NodeMetadata<'x>
        where
            Self: 'x;

        type GroupRef<'x>
            = FlattenByKeyGroup<'x, H, F>
        where
            Self: 'x;

        type LeafMetadata<'x>
            = H::LeafMetadata<'x>
        where
            Self: 'x;

        type GroupMetadata<'x>
            = H::GroupMetadata<'x>
        where
            Self: 'x;

        type DoesLoading = H::DoesLoading;

        fn root(&self) -> Self::GroupRef<'_> {
            FlattenByKeyGroup {
                this: self.0.root(),
                map_fn: &self.1,
            }
        }

        fn reborrow_groupref<'long: 'short, 'short>(
            it: FlattenByKeyGroup<'long, H, F>,
        ) -> FlattenByKeyGroup<'short, H, F> {
            FlattenByKeyGroup {
                this: H::reborrow_groupref(it.this),
                map_fn: it.map_fn,
            }
        }
    }
}
