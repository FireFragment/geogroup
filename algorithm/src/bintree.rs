use crate::deep_sorter::FirstLast;

use super::*;
use std::borrow::Cow;
use std::cell::OnceCell;
use std::fmt;
use std::hash::Hash;
use std::sync::OnceLock;
use std::{
    convert::Infallible,
    iter,
    ops::{Index, IndexMut},
};

use geogroup_common::SortableItem;
use lazy_hierarchy::AsGroupRef;

use deep_sorter::StaticNodeInfo;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum HorizontalIdx {
    Left = 0,
    Right = 1,
}

pub enum BinTree<Item: SortableItem, NDItem, NameErr, NodeInfoPresent: TyBool = TyTrue> {
    // IMPORTANT: When changing fields, make sure to update Debug impl
    InnerNode(BTInnerNode<Item, NDItem, NameErr, NodeInfoPresent>),
    Leaf(
        Item,
        TyOption<StaticNodeInfo<Item, NDItem, NameErr, Infallible>, NodeInfoPresent>,
    ),
}

impl<Item: SortableItem, NDItem, NameErr, NodeInfoPresent: TyBool> IndexMut<HorizontalIdx>
    for BTInnerNode<Item, NDItem, NameErr, NodeInfoPresent>
{
    fn index_mut(&mut self, index: HorizontalIdx) -> &mut Self::Output {
        match index {
            HorizontalIdx::Left => &mut self.positioned_children[0],
            HorizontalIdx::Right => &mut self.positioned_children[1],
        }
    }
}

impl<Item: SortableItem, NDItem, NameErr, NodeInfoPresent: TyBool> Index<HorizontalIdx>
    for BTInnerNode<Item, NDItem, NameErr, NodeInfoPresent>
{
    type Output = BinTree<Item, NDItem, NameErr, NodeInfoPresent>;
    fn index(&self, index: HorizontalIdx) -> &Self::Output {
        match index {
            HorizontalIdx::Left => &self.positioned_children[0],
            HorizontalIdx::Right => &self.positioned_children[1],
        }
    }
}

/*impl<Leaf, InnerNode, Node> Index<HorizontalIdx> for BTInnerNode<Leaf, InnerNode, Node> {
    type Output = BinTree<Leaf, InnerNode, Node>;

    fn index(&self, index: HorizontalIdx) -> &Self::Output {
        match index {
            HorizontalIdx::Left => &self.positioned_children[0],
            HorizontalIdx::Right => &self.positioned_children[1],
        }
    }
}

impl<Leaf, InnerNode, Node> IndexMut<HorizontalIdx> for BTInnerNode<Leaf, InnerNode, Node> {
    fn index_mut(&mut self, index: HorizontalIdx) -> &mut Self::Output {
        match index {
            HorizontalIdx::Left =>  &mut self.positioned_children[0],
            HorizontalIdx::Right => &mut self.positioned_children[1],
        }
    }
}*/

// impl<Leaf: std::fmt::Debug, InnerNode: std::fmt::Debug, Node: std::fmt::Debug> std::fmt::Debug
//     for BinTree<Leaf, InnerNode, Node>
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::InnerNode(arg0) => f.debug_tuple("InnerNode").field(arg0).finish(),
//             Self::Leaf(arg0, arg1) => f.debug_tuple("Leaf").field(arg0).field(arg1).finish(),
//         }
//     }
// }

impl<Item: SortableItem, NDItem, NameErr> BinTree<Item, NDItem, NameErr> {
    pub fn get_node_info(&self) -> &StaticNodeInfo<Item, NDItem, NameErr> {
        match self {
            BinTree::InnerNode(node) => &node.node_data,
            BinTree::Leaf(_, node) => node,
        }
    }
}

impl<Item: SortableItem, NDItem, NameErr, NodeInfoPresent: TyBool>
    BinTree<Item, NDItem, NameErr, NodeInfoPresent>
{
    pub fn get_leftmost_item(&self) -> &Item {
        match self {
            BinTree::InnerNode(node) => node.positioned_children[0].get_leftmost_item(),
            BinTree::Leaf(leaf, node) => leaf,
        }
    }
}

pub struct BTInnerNode<Item: SortableItem, NDItem, NameErr, NodeInfoPresent: TyBool = TyTrue> {
    // IMPORTANT: When changing fields, make sure to update Debug impl
    pub positioned_children: Box<[BinTree<Item, NDItem, NameErr, NodeInfoPresent>; 2]>,
    pub node_data: TyOption<StaticNodeInfo<Item, NDItem, NameErr, Infallible>, NodeInfoPresent>,
    /// For those items, [`StaticNodeInfo`] is generated lazily on-demand if needed
    pub additional_middle_leaves: Vec<Item>,
}

/// [`lazy_hierarchy::LeafRef`] implementation for [`BinTree`]
#[derive(Debug)]
pub struct BTLeafRef<'a, L, N>(&'a L, MaybeBorrowed<'a, N>);

impl<'a, L, N: Clone> Clone for BTLeafRef<'a, L, N> {
    fn clone(&self) -> Self {
        BTLeafRef(self.0, self.1.clone())
    }
}

impl<'a, L, N: Clone> lazy_hierarchy::LeafRef for BTLeafRef<'a, L, N> {
    type LeafData = &'a L;

    type NodeData = MaybeBorrowed<'a, N>;

    fn leaf_data(&self) -> Self::LeafData {
        &self.0
    }

    fn node_data(&self) -> Self::NodeData {
        self.1.clone()
    }
}

impl<'a, Item: SortableItem, NDItem, NameErr> lazy_hierarchy::GroupRef
    for &'a BTInnerNode<Item, NDItem, NameErr>
where
    StaticNodeInfo<Item, NDItem, NameErr, Item::PositionErr>: Clone,
    StaticNodeInfo<Item, NDItem, NameErr>: Clone,
{
    fn get_children(
        &self,
    ) -> Result<impl Iterator<Item = lazy_hierarchy::NodeRef<Self>>, Infallible> {
        let [child_1, child_2] = self.positioned_children.each_ref().map(|node| match node {
            BinTree::InnerNode(group) => lazy_hierarchy::NodeRef::Group(group),
            BinTree::Leaf(l, n) => lazy_hierarchy::NodeRef::Leaf(BTLeafRef(
                l,
                MaybeBorrowed::Owned(n.clone().uwnrap_infallible().generalize_err()),
            )),
        });
        // Also with noda data
        let middle_leaves = self.additional_middle_leaves.iter().map(|leaf| {
            lazy_hierarchy::NodeRef::Leaf(BTLeafRef(
                leaf,
                MaybeBorrowed::Owned(StaticNodeInfo {
                    fl_time: deep_sorter::FirstLast::new_single(leaf.get_time()),
                    fl_pos: leaf.get_position().map(|loc| {
                        debug_assert!(false, "Middle leaf suddenly has a valid location");
                        FirstLast::new_single(loc)
                    }),
                    naming_data: OnceLock::new(),
                }),
            ))
        });

        Ok(iter::once(child_1)
            .chain(middle_leaves)
            .chain(iter::once(child_2)))
    }

    fn group_data(&self) -> () {}

    fn node_data(&self) -> Self::NodeData {
        MaybeBorrowed::Owned((*self.node_data).clone().generalize_err())
    }

    type NodeData = MaybeBorrowed<'a, StaticNodeInfo<Item, NDItem, NameErr, Item::PositionErr>>; // TODO: Remove MaybeBorrowed, it's all owned anyways
    type LeafData = &'a Item;
    type GroupData = ();
    type StructureErr = Infallible;
    type LeafRef = BTLeafRef<'a, Item, StaticNodeInfo<Item, NDItem, NameErr, Item::PositionErr>>;
}

impl<Item: SortableItem, NDItem: Clone, NameErr: Clone> AsGroupRef for BinTree<Item, NDItem, NameErr>
where
    StaticNodeInfo<Item, NDItem, NameErr, Item::PositionErr>: Clone,
{
    type GroupRef<'a>
        = &'a BTInnerNode<Item, NDItem, NameErr>
    where
        Self: 'a;

    fn root(&self) -> Self::GroupRef<'_> {
        match self {
            BinTree::Leaf(_, _) => todo!(),
            BinTree::InnerNode(group) => group,
        }
    }
}

pub use bloat::*;
pub mod bloat {
    use super::*;

    impl<
            Item: SortableItem<Position = impl fmt::Debug, Time = impl fmt::Debug> + fmt::Debug,
            NDItem: fmt::Debug,
            NameErr: fmt::Debug,
        > fmt::Debug for BinTree<Item, NDItem, NameErr>
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::InnerNode(arg0) => f.debug_tuple("InnerNode").field(arg0).finish(),
                Self::Leaf(arg0, arg1) => f.debug_tuple("Leaf").field(arg0).field(arg1).finish(),
            }
        }
    }

    impl<
            Item: SortableItem<Position = impl fmt::Debug, Time = impl fmt::Debug> + fmt::Debug,
            NDItem: fmt::Debug,
            NameErr: fmt::Debug,
        > fmt::Debug for BTInnerNode<Item, NDItem, NameErr>
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("BTInnerNode")
                .field("positioned_children", &self.positioned_children)
                .field("node_data", &self.node_data)
                .field("additional_middle_leaves", &self.additional_middle_leaves)
                .finish()
        }
    }
}
