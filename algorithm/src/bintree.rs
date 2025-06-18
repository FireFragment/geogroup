use std::convert::Infallible;

use geogroup_common::hiearchy;

pub enum BinTree<Leaf, InnerNode> {
    InnerNode(BTInnerNode<Leaf, InnerNode>),
    Leaf(Leaf),
}

/*impl<Leaf, InnerNode> BinTree<Leaf, InnerNode> {
    pub type InnerNode = BTInnerNode<Leaf, InnerNode>;
}*/

pub struct BTInnerNode<Leaf, InnerNode> {
    pub children: Box<[BinTree<Leaf, InnerNode>; 2]>,
    pub data: InnerNode,
}

/// [`hiearchy::lazy::LeafRef`] implementation for [`BinTree`]
pub struct BTLeafRef<'a, L>(&'a L);

impl<'a, L> hiearchy::lazy::LeafRef for BTLeafRef<'a, L> {
    type Metadata = &'a L;

    type NodeMetadata = ();

    fn leaf_metadata(&self) -> Self::Metadata {
        &self.0
    }

    fn node_metadata(&self) -> Self::NodeMetadata {}
}

impl<'a, Leaf, InnerNode> hiearchy::lazy::GroupRef<'a> for &'a BTInnerNode<Leaf, InnerNode> {
    type Hiearchy = BinTree<Leaf, InnerNode>;

    fn get_children<'b>(
        &self,
    ) -> hiearchy::lazy::LoadingResult<
        impl Iterator<Item = hiearchy::lazy::NodeRef<'b, Self::Hiearchy>>,
        <Self::Hiearchy as hiearchy::Lazy>::StructureErr,
        <Self::Hiearchy as hiearchy::Lazy>::DoesLoading,
    >
    where
        'a: 'b,
    {
        hiearchy::lazy::LoadingResult::new_ok(self.children.iter().map(|node| match node {
            BinTree::InnerNode(group) => hiearchy::lazy::NodeRef::Group(group),
            BinTree::Leaf(l) => hiearchy::lazy::NodeRef::Leaf(BTLeafRef(l)),
        }))
    }

    fn group_metadata<'b>(&self) -> <Self::Hiearchy as hiearchy::Lazy>::GroupMetadata<'b>
    where
        'a: 'b,
    {
        &self.data
    }

    fn node_metadata<'b>(&self) -> <Self::Hiearchy as hiearchy::Lazy>::NodeMetadata<'b>
    where
        'a: 'b,
    {
    }
}

impl<Leaf, InnerNode> hiearchy::Lazy for BinTree<Leaf, InnerNode> {
    type LeafRef<'a>
        = BTLeafRef<'a, Leaf>
    where
        Self: 'a;

    type GroupRef<'a>
        = &'a BTInnerNode<Leaf, InnerNode>
    where
        Self: 'a;

    type NodeMetadata<'a>
        = ()
    where
        Self: 'a;

    type LeafMetadata<'a>
        = &'a Leaf
    where
        Self: 'a;

    type StructureErr = Infallible;

    type GroupMetadata<'a>
        = &'a InnerNode
    where
        Self: 'a;

    type DoesLoading = Infallible;

    fn root(&self) -> Self::GroupRef<'_> {
        match self {
            BinTree::Leaf(_) => todo!(),
            BinTree::InnerNode(group) => group,
        }
    }

    fn reborrow_groupref<'long: 'short, 'short>(
        it: Self::GroupRef<'long>,
    ) -> Self::GroupRef<'short> {
        it
    }
}
