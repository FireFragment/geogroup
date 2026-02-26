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
        /// Id unique in the entire tree.
        u64,
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
            BinTree::Leaf(_, _, node) => node,
        }
    }
}

impl<Item: SortableItem, NDItem, NameErr, NodeInfoPresent: TyBool>
    BinTree<Item, NDItem, NameErr, NodeInfoPresent>
{
    pub fn get_leftmost_item(&self) -> &Item {
        match self {
            BinTree::InnerNode(node) => node.positioned_children[0].get_leftmost_item(),
            BinTree::Leaf(leaf, _, node) => leaf,
        }
    }
}

pub struct BTInnerNode<Item: SortableItem, NDItem, NameErr, NodeInfoPresent: TyBool = TyTrue> {
    // IMPORTANT: When changing fields, make sure to update Debug impl
    pub positioned_children: Box<[BinTree<Item, NDItem, NameErr, NodeInfoPresent>; 2]>,
    pub node_data: TyOption<StaticNodeInfo<Item, NDItem, NameErr, Infallible>, NodeInfoPresent>,
    /// For those items, [`StaticNodeInfo`] is generated lazily on-demand if needed.
    /// The second element is the ID of the item.
    pub additional_middle_leaves: Vec<(Item, u64)>,
    /// Static group ID unique in the entire tree.
    pub node_id: u64,
}

/// [`lazy_hierarchy::LeafRef`] implementation for [`BinTree`]
pub struct BTLeafRef<'a, Item: SortableItem, NDItem, NameErr>(
    &'a Item,
    deep_sorter::NodeInfo<'a, Item, NDItem, NameErr>
);

impl<'a, Item: SortableItem + fmt::Debug, NDItem: fmt::Debug, NameErr: fmt::Debug> fmt::Debug for BTLeafRef<'a, Item, NDItem, NameErr>
where
    <Item as geogroup_common::SortableItem>::Time: std::fmt::Debug,
    <Item as geogroup_common::SortableItem>::Position: std::fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_tuple("BTLeafRef").field(&self.0).field(&self.1).finish()
    }
}

impl<'a, Item: SortableItem, NDItem, NameErr> Clone for BTLeafRef<'a, Item, NDItem, NameErr> {
    fn clone(&self) -> Self {
        BTLeafRef(self.0, self.1.clone())
    }
}

impl<'a, Item: SortableItem, NDItem, NameErr> lazy_hierarchy::LeafRef for BTLeafRef<'a, Item, NDItem, NameErr> {
    type LeafData = &'a Item;

    type NodeData = deep_sorter::NodeInfo<'a, Item, NDItem, NameErr>;

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
            BinTree::Leaf(l, id, n) => {
                lazy_hierarchy::NodeRef::Leaf(
                    BTLeafRef(
                        l,
                        deep_sorter::NodeInfo::new(
                            Some(n.as_ref().uwnrap_infallible()),
                            *id
                        )
                    ))
            }
        });
        // Also with noda data
        let middle_leaves = self
            .additional_middle_leaves
            .iter()
            .map(|leaf| lazy_hierarchy::NodeRef::Leaf(BTLeafRef(&leaf.0,
                deep_sorter::NodeInfo::new(None, leaf.1))));

        Ok(iter::once(child_1)
            .chain(middle_leaves)
            .chain(iter::once(child_2)))
    }

    fn group_data(&self) -> () {}

    fn node_data(&self) -> Self::NodeData {
        deep_sorter::NodeInfo::new(
            Some(self.node_data.as_ref().uwnrap_infallible()),
            self.node_id
        )
    }

    type NodeData = deep_sorter::NodeInfo<'a, Item, NDItem, NameErr>; // TODO: Remove MaybeBorrowed, it's all owned anyways
    type LeafData = &'a Item;
    type GroupData = ();
    type StructureErr = Infallible;
    type LeafRef = BTLeafRef<'a, Item, NDItem, NameErr>;
}

impl<Item: SortableItem, NDItem: Clone, NameErr: Clone> AsGroupRef
    for BinTree<Item, NDItem, NameErr>
where
    StaticNodeInfo<Item, NDItem, NameErr, Item::PositionErr>: Clone,
{
    type GroupRef<'a>
        = &'a BTInnerNode<Item, NDItem, NameErr>
    where
        Self: 'a;

    fn root(&self) -> Self::GroupRef<'_> {
        match self {
            BinTree::Leaf(_, _, _) => todo!(),
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
                Self::Leaf(arg0, arg1, arg2) => f.debug_tuple("Leaf").field(arg0).field(arg1).field(arg2).finish(),
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
