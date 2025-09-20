use std::{convert::Infallible, ops::{Index, IndexMut}};

use lazy_hierarchy::AsGroupRef;

#[derive(Debug, Clone, Hash)]
pub enum HorizontalIdx {
    Left, Right
}

pub enum BinTree<Leaf, InnerNode, Node> {
    InnerNode(BTInnerNode<Leaf, InnerNode, Node>),
    Leaf(Leaf, Node),
}

impl<Leaf, InnerNode, Node> Index<HorizontalIdx> for BTInnerNode<Leaf, InnerNode, Node> {
    type Output = BinTree<Leaf, InnerNode, Node>;

    fn index(&self, index: HorizontalIdx) -> &Self::Output {
        match index {
            HorizontalIdx::Left => &self.children[0],
            HorizontalIdx::Right => &self.children[1],
        }
    }
}

impl<Leaf, InnerNode, Node> IndexMut<HorizontalIdx> for BTInnerNode<Leaf, InnerNode, Node> {
    fn index_mut(&mut self, index: HorizontalIdx) -> &mut Self::Output {
        match index {
            HorizontalIdx::Left => &mut self.children[0],
            HorizontalIdx::Right => &mut self.children[1],
        }
    }
}


impl<Leaf: std::fmt::Debug, InnerNode: std::fmt::Debug, Node: std::fmt::Debug> std::fmt::Debug
    for BinTree<Leaf, InnerNode, Node>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InnerNode(arg0) => f.debug_tuple("InnerNode").field(arg0).finish(),
            Self::Leaf(arg0, arg1) => f.debug_tuple("Leaf").field(arg0).field(arg1).finish(),
        }
    }
}

impl<Leaf, InnerNode, Node> BinTree<Leaf, InnerNode, Node> {
    pub fn get_node_data(&self) -> &Node {
        match self {
            BinTree::InnerNode(node) => &node.node_data,
            BinTree::Leaf(_, node) => node,
        }
    }
}

#[derive(Debug)]
pub struct BTInnerNode<Leaf, InnerNode, Node> {
    pub children: Box<[BinTree<Leaf, InnerNode, Node>; 2]>,
    pub inner_node_data: InnerNode,
    pub node_data: Node,
}

/// [`lazy_hierarchy::LeafRef`] implementation for [`BinTree`]
#[derive(Debug)]
pub struct BTLeafRef<'a, L, N>(&'a L, &'a N);

impl<'a, L, N> Clone for BTLeafRef<'a, L, N> {
    fn clone(&self) -> Self {
        BTLeafRef(self.0, self.1)
    }
}

impl<'a, L, N> lazy_hierarchy::LeafRef for BTLeafRef<'a, L, N> {
    type LeafData = &'a L;

    type NodeData = &'a N;

    fn leaf_data(&self) -> Self::LeafData {
        &self.0
    }

    fn node_data(&self) -> Self::NodeData {
        &self.1
    }
}

impl<'a, Leaf, InnerNode, Node> lazy_hierarchy::GroupRef
    for &'a BTInnerNode<Leaf, InnerNode, Node>
{
    fn get_children(
        &self,
    ) -> Result<impl Iterator<Item = lazy_hierarchy::NodeRef<Self>>, Infallible> {
        Ok(self.children.iter().map(|node| match node {
            BinTree::InnerNode(group) => lazy_hierarchy::NodeRef::Group(group),
            BinTree::Leaf(l, n) => lazy_hierarchy::NodeRef::Leaf(BTLeafRef(l, n)),
        }))
    }

    fn group_data(&self) -> &'a InnerNode {
        &self.inner_node_data
    }

    fn node_data(&self) -> &'a Node {
        &self.node_data
    }

    type NodeData = &'a Node;
    type LeafData = &'a Leaf;
    type GroupData = &'a InnerNode;
    type StructureErr = Infallible;
    type LeafRef = BTLeafRef<'a, Leaf, Node>;
}

impl<Leaf, InnerNode, Node> AsGroupRef for BinTree<Leaf, InnerNode, Node> {
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
