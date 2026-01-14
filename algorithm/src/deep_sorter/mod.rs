pub mod naming;
mod onetime;
//#[cfg(test)]
//mod test;

use core::fmt;
use std::{
    cell::OnceCell,
    convert::Infallible,
    ops::{Div, RangeInclusive},
    sync::OnceLock,
};

use crate::*;
use itertools::Itertools;
pub use onetime::*;
use std::hash::Hash;
use unwrap_infallible::UnwrapInfallible as _;

pub struct DeepSorter<Item: SortableItem, NDItem: Clone + PartialEq + Eq + Hash, NameErr> {
    bintree: BinTree<Item, NDItem, NameErr, TyTrue>,
}

impl<
        Item: SortableItem + fmt::Debug,
        NDItem: fmt::Debug + Clone + PartialEq + Eq + Hash,
        NameErr: fmt::Debug,
    > fmt::Debug for DeepSorter<Item, NDItem, NameErr>
where
    Item::Time: fmt::Debug,
    Item::Position: fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeepSorter")
            .field("bintree", &self.bintree)
            .finish()
    }
}

/// Returns a result between -1 and 1. To convert it to [`Strength`], multiply it with [`Strength::MAX`]
///
/// This formula for strength is used because
///  1) It makes any ratio representable in i16 thanks to `tanh` which bounds the result to
///     range (-1;1)
///  2) It's "symmetric": `ratio_to_strength(parent_separation, self_separation) == -ratio_to_strength(self_separation, parent_separation)`
pub fn ratio_to_strength(parent_separation: f32, self_separation: f32) -> f32 {
    // NOTE: Division by zero is OK here, because 1/0 = Infinity and Infinity.tanh() = 1
    (parent_separation / self_separation)
        .log2()
        .div(2.0) // This is a magic number, the strengths just seemed right with this
        .tanh()
}

/// # Sorting methods
impl<Item: SortableItem, NDItem: Clone + PartialEq + Eq + Hash, NameErr: Clone>
    DeepSorter<Item, NDItem, NameErr>
{
    pub fn deep_hierarchy<'s>(
        &'s self,
    ) -> impl lazy_hierarchy::GroupRef<
        GroupData = GroupInfo,
        LeafData = &'s Item,
        NodeData = NodeInfo<'s, Item, NDItem, NameErr>,
        StructureErr = Infallible,
        //LeafRef = impl Send,
    > + 's {
        self.bintree
            .root()
            // TODO: Consider isolating these computations somewhere else, eg. to `geogroup.rs` or `strength.rs`
            // Calculate separation
            .map_group_data(|group_ref| {
                let separation = group_ref
                    .get_children()
                    .unwrap_or_else(|e| match e {})
                    .filter_map(|child| child.node_data().fl_pos_opt().cloned())
                    .tuple_windows()
                    .map(|(item1, item2)| item1.last.distance(&item2.first))
                    .max();

                if separation.is_none() {
                    debug_assert!(
                        group_ref
                            .get_children()
                            .unwrap_or_else(|e| match e {})
                            .collect_vec()
                            .len()
                            < 2,
                    );
                }

                separation
            })
            .with_parent()
            // Calculate strength
            .map_group_data(|group_ref| {
                let self_separation = group_ref.group_data();
                let Some(parent) = group_ref.node_data().parent else {
                    return GroupInfo {
                        separation: self_separation,
                        strength: StrengthInfo::Root,
                    };
                };
                let parent_separation = parent.group_data();

                GroupInfo {
                    separation: self_separation,
                    strength: if let Some(self_separation) = self_separation {
                        if let Some(parent_separation) = parent_separation {
                            StrengthInfo::Ok {
                                strength: (ratio_to_strength(
                                    parent_separation as f32,
                                    self_separation as f32,
                                ) * Strength::MAX as f32)
                                    as Strength,
                                separation_ratio: parent_separation as f32 / self_separation as f32,
                            }
                        } else {
                            StrengthInfo::NoSiblings
                        }
                    } else {
                        StrengthInfo::LessThan2ChildrenWithLocation
                    },
                }
            })
            .with_idx()
            // "Revert" with_parent and add index to
            .map_node_data(|node_ref| {
                let node_data = node_ref.node_data();
                NodeInfo {
                    static_info: node_data.data.data,
                    id: match node_data.index {
                        0 => HorizontalIdx::Left,
                        1 => HorizontalIdx::Right,
                        idx => {
                            // TODO: This CAN happen when there're items in the middle
                            debug_assert!(false, "Index bigger than 1 in binary tree: {idx}");
                            HorizontalIdx::Right
                        }
                    },
                }
            })
        //
    }
}

/// Information about nodes which is generated once and then used instead of being generated on the fly
///
/// `PosErr` is used mainly as a way to sometimes make sure that first and last position is included, so it's
/// often set to [`Infallible`]
pub struct StaticNodeInfo<Item: SortableItem, NDItem, NameErr, PosErr = Infallible> {
    /// `fl` means "first and last" here
    pub fl_time: FirstLast<Item::Time>,
    /// `fl` means "first and last" here
    /// It's recommended to access this using the `fl_pos` method instead of through the field
    pub fl_pos: Result<FirstLast<Item::Position>, PosErr>,

    pub naming_data: OnceLock<Result<Vec<NDItem>, NameErr>>,
}

impl<Item: SortableItem, NDItem, NameErr> StaticNodeInfo<Item, NDItem, NameErr, Infallible> {
    pub fn fl_pos(&self) -> &FirstLast<Item::Position> {
        self.fl_pos.as_ref().map_err(|e| *e).unwrap_infallible()
    }

    pub fn generalize_err<E>(self) -> StaticNodeInfo<Item, NDItem, NameErr, E> {
        StaticNodeInfo {
            fl_time: self.fl_time,
            fl_pos: self.fl_pos.map_err(|e| match e {}),
            naming_data: self.naming_data,
        }
    }
}

impl<Item: SortableItem, NDItem, NameErr, PosErr> StaticNodeInfo<Item, NDItem, NameErr, PosErr> {
    pub fn fl_pos_opt(&self) -> Option<&FirstLast<Item::Position>> {
        self.fl_pos.as_ref().ok()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct FirstLast<T> {
    pub first: T,
    pub last: T,
}

impl<T: Clone> FirstLast<T> {
    pub fn new_single(t: T) -> Self {
        Self {
            first: t.clone(),
            last: t,
        }
    }
}

pub struct NodeInfo<'hier, Item: SortableItem, NDItem, NameErr> {
    static_info: MaybeBorrowed<'hier, StaticNodeInfo<Item, NDItem, NameErr, Item::PositionErr>>,
    pub id: HorizontalIdx,
}

impl<'hier, Item: SortableItem, NDItem: Clone, NameErr: Clone>
    NodeInfo<'hier, Item, NDItem, NameErr>
{
    /// Returns [`None`] if the item has not yet been assigned a name,
    /// returns `Some(Err)` it there was a failed attempt to assign a name
    pub fn get_naming_data<'s: 'hier>(&'s self) -> Option<Result<Vec<NDItem>, NameErr>> {
        self.static_info.naming_data.get().cloned()
    }
}

impl<Item: SortableItem, NDItem: Clone, NameErr: Clone, PosErr: Clone> Clone
    for StaticNodeInfo<Item, NDItem, NameErr, PosErr>
{
    fn clone(&self) -> Self {
        Self {
            fl_time: self.fl_time.clone(),
            fl_pos: self.fl_pos.clone(),
            naming_data: self.naming_data.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StrengthInfo {
    /// Strength couldn't be determined because this group has less than two children with known location.
    LessThan2ChildrenWithLocation,
    /// Strength couldn't be determined because this group has no siblings (ie. its parent has only one child)
    /// AND this group has 2 children or more (if it had less, this would be [`StrengthInfo::LessThan2Children`])
    NoSiblings,
    /// Strength couldn't be determined for root groups and this _is_ a root group.
    Root,
    Ok {
        strength: Strength,
        separation_ratio: f32,
    },
}

#[derive(Clone, Debug)]
pub struct GroupInfo {
    pub strength: StrengthInfo,
    /// If [None], the group has less than two children
    pub separation: Option<Distance>,
}

/// Provides [NodeInfo] for every node in the binary tree
///
/// Expects that all leaves in the binary tree
/// (but not in [`additional_middle_leaves`](BTInnerNode::additional_middle_leaves) contain location - otherwise panics.
fn provide_info<Item: SortableItem, NDItem, NameErr>(
    tree: BinTree<Item, NDItem, NameErr, TyFalse>,
) -> BinTree<Item, NDItem, NameErr, TyTrue> {
    match tree {
        BinTree::InnerNode(node) => {
            let children = Box::new(node.positioned_children.map(|c| provide_info(c)));
            BinTree::InnerNode(BTInnerNode {
                node_data: StaticNodeInfo {
                    naming_data: OnceLock::new(),
                    fl_time: FirstLast {
                        first: children
                            .first()
                            .unwrap()
                            .get_node_info()
                            .fl_time
                            .first
                            .clone(),
                        last: children
                            .last()
                            .unwrap()
                            .get_node_info()
                            .fl_time
                            .last
                            .clone(),
                    },
                    fl_pos: Ok(FirstLast {
                        first: children
                            .first()
                            .unwrap()
                            .get_node_info()
                            .fl_pos()
                            .first
                            .clone(),
                        last: children
                            .last()
                            .unwrap()
                            .get_node_info()
                            .fl_pos()
                            .last
                            .clone(),
                    }),
                }
                .into(),
                positioned_children: children,
                additional_middle_leaves: node.additional_middle_leaves,
            })
        }
        BinTree::Leaf(l, TyOption::Empty(())) => {
            let tree_group_info = StaticNodeInfo {
                fl_time: FirstLast::new_single(l.get_time()),
                fl_pos: Ok(FirstLast::new_single(l.get_position().expect("provide_info called with a tree which has item in the binary tree that hasn't position "))),
                naming_data: OnceLock::new(),
            };
            BinTree::Leaf(l, tree_group_info.into())
        }
    }
}

impl<Item: SortableItem, NDItem: Clone + PartialEq + Eq + Hash, NameErr>
    DeepSorter<Item, NDItem, NameErr>
{
    /// This sorts all items to binary tree, potentially long-running
    pub fn new(items: Vec<Item>) -> Self {
        //points.sort_unstable_by_key(|it| it.get_time());

        Self {
            bintree: provide_info(sort_to_binary_tree(items)),
        }
    }

    //pub fn add_items(&mut self, item: impl IntoIterator<Item = Item>) { todo!() }

    // /// Get the smallest group which still encapsulates a given time
    // fn get_time_group_mut(
    //     &mut self,
    //     time: &Item::Time,
    // ) -> Option<&mut BTInnerNode<Item, (), StaticNodeInfo<Item, NDItem, NameErr>>> {
    //     get_time_group_mut(&mut self.bintree, time)
    // }
}

/// Check if a [`BinTree`] contains the given time.
///
/// Same behavior as `get_time_group_mut.is_some()`
///
/// Returns false if the `tree` itself is out of range, that is if there are either
/// no newer items than `time` OR if there are no older items than `time`.
/// This means that for `BinTree::Leaf` we always return [false].
fn tree_contains_time<Item: SortableItem, NDItem, NameErr>(
    tree: &BinTree<Item, (), StaticNodeInfo<Item, NDItem, NameErr>>,
    time: &Item::Time,
) -> bool {
    match tree {
        BinTree::Leaf(_, _) => false,
        BinTree::InnerNode(node) => {
            !(node.node_data.fl_time.first > *time || node.node_data.fl_time.last < *time)
        }
    }
}

/// Get the smallest group which still contains a given time
///
/// Returns [None] if the `tree` itself is out of range, that is if there are either
/// no newer items than `time` OR if there are no older items than `time`.
/// This means that for `BinTree::Leaf` we always return [None].
///
/// Returns [None] if and only if the [`tree_contains_time`] returns [`false`] for the same input.
///
/// # Panics
///
/// May panic on invalid tree, eg. invalid values //TODO: What does this mean??
fn get_time_group_mut<'a, Item: SortableItem, NDItem, NameErr>(
    tree: &'a mut BinTree<Item, (), StaticNodeInfo<Item, NDItem, NameErr>>,
    time: &Item::Time,
) -> Option<&'a mut BTInnerNode<Item, (), StaticNodeInfo<Item, NDItem, NameErr>>> {
    match tree {
        BinTree::Leaf(_, _) => None,
        BinTree::InnerNode(node) => {
            if node.node_data.fl_time.first > *time || node.node_data.fl_time.last < *time {
                return None;
            }
            // The following code is a bit dirty, because using just recursion into this function and
            // checking for `None` leads to borrow checker crying

            // First check if left child contains a more specific group
            if tree_contains_time(&node.positioned_children[0], time) {
                return get_time_group_mut(&mut node.positioned_children[0], time);
            }

            // Then check if right child contains a more specific group
            if tree_contains_time(&node.positioned_children[1], time) {
                return get_time_group_mut(&mut node.positioned_children[1], time);
            }

            // If neither child contains it, this node is the smallest containing group
            Some(node)
        }
    }
}

pub struct GroupRef<'a, Item: SortableItem>(&'a [Item]);

#[non_exhaustive]
pub struct GroupMetadata {
    pub strength: u8,
}

impl<
        'hier,
        Item: SortableItem<Position = impl fmt::Debug, Time = impl fmt::Debug>,
        NDItem: fmt::Debug,
        NameErr: fmt::Debug,
    > fmt::Debug for NodeInfo<'hier, Item, NDItem, NameErr>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeInfo")
            .field("static_info", &self.static_info)
            .field("id", &self.id)
            .finish()
    }
}
impl<
        Item: SortableItem<Position = impl fmt::Debug, Time = impl fmt::Debug>,
        NDItem: fmt::Debug,
        NameErr: fmt::Debug,
        PosErr: fmt::Debug,
    > fmt::Debug for StaticNodeInfo<Item, NDItem, NameErr, PosErr>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticNodeInfo")
            .field("fl_time", &self.fl_time)
            .field("fl_pos", &self.fl_pos)
            .field("naming_data", &self.naming_data)
            .finish()
    }
}
