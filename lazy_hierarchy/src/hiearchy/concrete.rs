use crate::*;
use std::convert::Infallible;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Node<G, L, N> {
    Group(Group<G, L, N>),
    Leaf(Leaf<L, N>),
}

impl<G, L, N> Node<G, L, N> {
    pub fn new_group(g: Group<G, L, N>) -> Self {
        Node::Group(g)
    }

    pub fn new_leaf(l: Leaf<L, N>) -> Self {
        Node::Leaf(l)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Leaf<L, N> {
    leaf_data: L,
    node_data: N,
}

impl<L, N> Leaf<L, N> {
    pub fn new(leaf_data: L, node_data: N) -> Self {
        Self {
            leaf_data,
            node_data,
        }
    }

    pub fn leaf_data_mut(&mut self) -> &mut L {
        &mut self.leaf_data
    }

    pub fn node_data_mut(&mut self) -> &mut N {
        &mut self.node_data
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Group<G, L, N> {
    children: Vec<Node<G, L, N>>,
    group_data: G,
    node_data: N,
}

impl<G, L, N> Group<G, L, N> {
    pub fn new(children: Vec<Node<G, L, N>>, group_data: G, node_data: N) -> Self {
        Self {
            children,
            group_data,
            node_data,
        }
    }
    pub fn children_mut(&mut self) -> &mut Vec<Node<G, L, N>> {
        &mut self.children
    }

    pub fn group_data_mut(&mut self) -> &mut G {
        &mut self.group_data
    }

    pub fn node_data_mut(&mut self) -> &mut N {
        &mut self.node_data
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConcreteHiearchy<G, L, N> {
    root_group: Group<G, L, N>,
}

impl<G, L, N> ConcreteHiearchy<G, L, N> {
    pub fn new(root_group: Group<G, L, N>) -> Self {
        Self { root_group }
    }

    pub fn root_group(&self) -> &Group<G, L, N> {
        &self.root_group
    }

    pub fn root_group_mut(&mut self) -> &mut Group<G, L, N> {
        &mut self.root_group
    }
}

impl<L, G, N> AsGroupRef for ConcreteHiearchy<G, L, N> {
    type GroupRef<'a>
        = &'a Group<G, L, N>
    where
        Self: 'a;

    fn root(&self) -> Self::GroupRef<'_> {
        &self.root_group
    }
}

impl<'a, N, L> LeafRef for &'a Leaf<L, N> {
    type LeafData = &'a L;
    type NodeData = &'a N;

    fn leaf_data(&self) -> Self::LeafData {
        &self.leaf_data
    }

    fn node_data(&self) -> Self::NodeData {
        &self.node_data
    }
}

impl<'a, G, L, N> GroupRef for &'a Group<G, L, N> {
    type NodeData = &'a N;
    type LeafData = &'a L;
    type GroupData = &'a G;
    type StructureErr = Infallible;
    type LeafRef = &'a Leaf<L, N>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr> {
        Ok(self.children.iter().map(|node| match node {
            Node::Group(group) => NodeRef::Group(group),
            Node::Leaf(leaf) => NodeRef::Leaf(leaf),
        }))
    }

    fn group_data(&self) -> Self::GroupData {
        &self.group_data
    }

    fn node_data(&self) -> Self::NodeData {
        &self.node_data
    }
}
