//! Creates a new hiearchy where every node holds a [reference](GroupRef) to its parent.
//!
//! **Performance:** for every call to `leaf_metadata` and `group_metadata`, it also additionally calls `node_metadata`, regardless
//! of whether it's needed

use super::*;

pub fn map_group_data<
    'a,
    'orig_gr: 'a,
    OrigGr: AsGroupRef<'a> + 'a,
    GroupDataNew: 'a,
    F: Fn(<OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata) -> GroupDataNew + 'a,
>(
    gr: &'orig_gr OrigGr,
    fun: F,
) -> AsMappedGroupRef<
    'a,
    GroupDataNew,
    <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
    <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    OrigGr,
    impl Fn(
        <<OrigGr as AsGroupRef<'a>>::GroupRef as GroupRef<'a>>::GroupMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> GroupDataNew,
    impl Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
    impl Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    impl Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
> {
    AsMappedGroupRef {
        original: gr,
        map_groups_fn: move |gr, _| fun(gr),
        map_leaves_fn: move |l, _| l,
        map_groups_node_data_fn: move |_, n| n,
        map_leaves_node_data_fn: move |_, n| n,
    }
}

pub struct AsMappedGroupRef<
    'a,
    GroupDataNew: 'a,
    LeafDataNew: 'a,
    NodeDataNew: 'a,
    OrigGr: AsGroupRef<'a>,
    FnGroupData: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> GroupDataNew,
    FnLeafData: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> LeafDataNew,
    FnNodeDataGroup: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> NodeDataNew,
    FnNodeDataLeaf: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> NodeDataNew,
> {
    original: &'a OrigGr,
    map_groups_fn: FnGroupData,
    map_leaves_fn: FnLeafData,
    map_groups_node_data_fn: FnNodeDataGroup,
    map_leaves_node_data_fn: FnNodeDataLeaf,
}

impl<
        'a,
        GroupDataNew: 'a,
        LeafDataNew: 'a,
        NodeDataNew: 'a,
        OrigGr: AsGroupRef<'a>,
        FnGroupData: Fn(
                <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
                <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
            ) -> GroupDataNew
            + 'a,
        FnLeafData: Fn(
                <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
                <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
            ) -> LeafDataNew
            + 'a,
        FnNodeDataGroup: Fn(
                <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
                <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
            ) -> NodeDataNew
            + 'a,
        FnNodeDataLeaf: Fn(
                <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
                <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
            ) -> NodeDataNew
            + 'a,
    > AsGroupRef<'a>
    for AsMappedGroupRef<
        'a,
        GroupDataNew,
        LeafDataNew,
        NodeDataNew,
        OrigGr,
        FnGroupData,
        FnLeafData,
        FnNodeDataGroup,
        FnNodeDataLeaf,
    >
{
    type GroupRef = MappedGroupRef<
        'a,
        GroupDataNew,
        LeafDataNew,
        NodeDataNew,
        OrigGr,
        FnGroupData,
        FnLeafData,
        FnNodeDataGroup,
        FnNodeDataLeaf,
    >;

    fn root<'s: 'a>(&'s self) -> Self::GroupRef {
        MappedGroupRef {
            this: self.original.root(),
            root: &self,
        }
    }
}

pub struct MappedGroupRef<
    'a,
    GroupDataNew: 'a,
    LeafDataNew: 'a,
    NodeDataNew: 'a,
    OrigGr: AsGroupRef<'a>,
    FnGroupData: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> GroupDataNew,
    FnLeafData: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> LeafDataNew,
    FnNodeDataGroup: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> NodeDataNew,
    FnNodeDataLeaf: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> NodeDataNew,
> {
    this: OrigGr::GroupRef,
    root: &'a AsMappedGroupRef<
        'a,
        GroupDataNew,
        LeafDataNew,
        NodeDataNew,
        OrigGr,
        FnGroupData,
        FnLeafData,
        FnNodeDataGroup,
        FnNodeDataLeaf,
    >,
}

pub struct MappedLeafRef<
    'a,
    OrigGr: AsGroupRef<'a>,
    NewGr: GroupRef<'a>,
    FnLeafData: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> NewGr::LeafMetadata,
    FnNodeDataLeaf: Fn(
        <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> NewGr::NodeMetadata,
> {
    this: <OrigGr::GroupRef as GroupRef<'a>>::LeafRef,
    map_leaves_fn: &'a FnLeafData,
    map_leaves_node_data_fn: &'a FnNodeDataLeaf,
    phantom_data: PhantomData<&'a NewGr>,
}

impl<
        'a,
        OrigGr: AsGroupRef<'a>,
        NewGr: GroupRef<'a>,
        FnLeafData: Fn(
            <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
            <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
        ) -> NewGr::LeafMetadata,
        FnNodeDataLeaf: Fn(
            <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
            <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
        ) -> NewGr::NodeMetadata,
    > LeafRef for MappedLeafRef<'a, OrigGr, NewGr, FnLeafData, FnNodeDataLeaf>
{
    type Metadata = NewGr::LeafMetadata;
    type NodeMetadata = NewGr::NodeMetadata;

    fn leaf_metadata(&self) -> NewGr::LeafMetadata {
        (self.map_leaves_fn)(self.this.leaf_metadata(), self.this.node_metadata())
    }

    fn node_metadata(&self) -> Self::NodeMetadata {
        (self.map_leaves_node_data_fn)(self.this.leaf_metadata(), self.this.node_metadata())
    }
}

impl<
        'a,
        GroupDataNew: 'a,
        LeafDataNew: 'a,
        NodeDataNew: 'a,
        OrigGr: AsGroupRef<'a>,
        FnGroupData: Fn(
            <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
            <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
        ) -> GroupDataNew,
        FnLeafData: Fn(
            <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
            <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
        ) -> LeafDataNew,
        FnNodeDataGroup: Fn(
            <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
            <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
        ) -> NodeDataNew,
        FnNodeDataLeaf: Fn(
            <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
            <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
        ) -> NodeDataNew,
    > GroupRef<'a>
    for MappedGroupRef<
        'a,
        GroupDataNew,
        LeafDataNew,
        NodeDataNew,
        OrigGr,
        FnGroupData,
        FnLeafData,
        FnNodeDataGroup,
        FnNodeDataLeaf,
    >
{
    type NodeMetadata = NodeDataNew;
    type LeafMetadata = LeafDataNew;
    type GroupMetadata = GroupDataNew;
    type StructureErr = <OrigGr::GroupRef as GroupRef<'a>>::StructureErr;
    type LeafRef = MappedLeafRef<'a, OrigGr, Self, FnLeafData, FnNodeDataLeaf>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<'a, Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        self.this.get_children().map(move |iter| {
            iter.map(|child| match child {
                NodeRef::Group(group) => NodeRef::Group(MappedGroupRef {
                    this: group,
                    root: self.root,
                }),
                NodeRef::Leaf(leaf) => NodeRef::Leaf(MappedLeafRef {
                    this: leaf,
                    map_leaves_fn: &self.root.map_leaves_fn,
                    map_leaves_node_data_fn: &self.root.map_leaves_node_data_fn,
                    phantom_data: PhantomData,
                }),
            })
        })
    }

    fn group_metadata(&self) -> Self::GroupMetadata {
        (self.root.map_groups_fn)(self.this.group_metadata(), self.this.node_metadata())
    }

    fn node_metadata<'b>(&self) -> Self::NodeMetadata {
        (self.root.map_groups_node_data_fn)(self.this.group_metadata(), self.this.node_metadata())
    }
}
