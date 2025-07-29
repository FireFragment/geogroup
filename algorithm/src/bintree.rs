use std::convert::Infallible;

use geogroup_common::hiearchy;

pub enum BinTree<Leaf, InnerNode, Node> {
    InnerNode(BTInnerNode<Leaf, InnerNode, Node>),
    Leaf(Leaf, Node),
}

impl<Leaf, InnerNode, Node> BinTree<Leaf, InnerNode, Node> {
    pub fn get_node_data(&self) -> &Node {
        match self {
            BinTree::InnerNode(node) => &node.node_data,
            BinTree::Leaf(_, node) => node,
        }
    }
}

pub struct BTInnerNode<Leaf, InnerNode, Node> {
    pub children: Box<[BinTree<Leaf, InnerNode, Node>; 2]>,
    pub inner_node_data: InnerNode,
    pub node_data: Node,
}

/// [`hiearchy::lazy::LeafRef`] implementation for [`BinTree`]
pub struct BTLeafRef<'a, L, N>(&'a L, &'a N);

impl<'a, L, N> hiearchy::lazy::LeafRef for BTLeafRef<'a, L, N> {
    type Metadata = &'a L;

    type NodeMetadata = &'a N;

    fn leaf_metadata(&self) -> Self::Metadata {
        &self.0
    }

    fn node_metadata(&self) -> Self::NodeMetadata {
        &self.1
    }
}

impl<'a, Leaf, InnerNode, Node> hiearchy::lazy::GroupRef
    for &'a BTInnerNode<Leaf, InnerNode, Node>
{
    fn get_children(
        &self,
    ) -> Result<impl Iterator<Item = hiearchy::lazy::NodeRef<Self>>, Infallible> {
        Ok(self.children.iter().map(|node| match node {
            BinTree::InnerNode(group) => hiearchy::lazy::NodeRef::Group(group),
            BinTree::Leaf(l, n) => hiearchy::lazy::NodeRef::Leaf(BTLeafRef(l, n)),
        }))
    }

    fn group_metadata(&self) -> &'a InnerNode {
        &self.inner_node_data
    }

    fn node_metadata(&self) -> &'a Node {
        &self.node_data
    }

    type NodeMetadata = &'a Node;
    type LeafMetadata = &'a Leaf;
    type GroupMetadata = &'a InnerNode;
    type StructureErr = Infallible;
    type LeafRef = BTLeafRef<'a, Leaf, Node>;
}

impl<Leaf, InnerNode, Node> hiearchy::Lazy for BinTree<Leaf, InnerNode, Node> {
    type GroupRef<'a>
        = &'a BTInnerNode<Leaf, InnerNode, Node>
    where
        Self: 'a;

    fn root(&self) -> Self::GroupRef<'_> {
        match self {
            BinTree::Leaf(_, _) => todo!(),
            BinTree::InnerNode(group) => group,
        }
    }
}
