mod onetime;
#[cfg(test)]
mod test;

use std::convert::Infallible;

use crate::*;
use geogroup_common::prelude::*;
use itertools::Itertools;
pub use onetime::*;

pub struct DeepSorter<Item: SortableItem> {
    bintree: BinTree<Item, (), NodeInfo<Item>>,
    params: Params,
}

impl<Item: SortableItem> DeepSorter<Item> {
    pub fn root<'s>(
        &'s self,
    ) -> impl hiearchy::lazy::GroupRef<
        GroupMetadata = GroupInfo,
        LeafMetadata = &'s Item,
        NodeMetadata = NodeInfo<Item>,
        StructureErr = Infallible,
    >
           + 's
           + use<'s, Item> {
        self.bintree
            .root()
            .map_group_data(|group_ref| {
                let separation = group_ref
                    .get_children()
                    .unwrap_or_else(|e| match e {})
                    .map(|child| child.node_data())
                    .tuple_windows()
                    .map(|(first, second)| first.last_pos.distance(&second.first_pos))
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
            .map_group_data(|group_ref| {
                let self_separation = group_ref.group_metadata();
                let Some(parent) = group_ref.node_metadata().parent else {
                    return GroupInfo {
                        separation: self_separation,
                        strength: StrengthInfo::Root,
                    };
                };
                let parent_separation = parent.group_metadata();

                GroupInfo {
                    separation: self_separation,
                    strength: if let Some(self_separation) = self_separation {
                        if let Some(parent_separation) = parent_separation {
                            // TODO: Prevent division by zero
                            StrengthInfo::Ok(parent_separation as f32 / self_separation as f32)
                        } else {
                            StrengthInfo::NoSiblings
                        }
                    } else {
                        StrengthInfo::LessThan2Children
                    },
                }
            })
            .map_node_data(|node_ref| node_ref.node_data().data.to_owned())
        //
    }
}

pub struct NodeInfo<Item: SortableItem> {
    first_time: Item::Time,
    last_time: Item::Time,

    first_pos: Item::Position,
    last_pos: Item::Position,
}

impl<Item: SortableItem> Clone for NodeInfo<Item> {
    fn clone(&self) -> Self {
        Self {
            first_time: self.first_time.clone(),
            last_time: self.last_time.clone(),
            first_pos: self.first_pos.clone(),
            last_pos: self.last_pos.clone(),
        }
    }
}

/// Strength lower than 1.0 means that the group has even higher "separation" than its parent.
pub type Strength = f32;
#[derive(Debug, Clone, PartialEq)]
pub enum StrengthInfo {
    /// Strength couldn't be determined because this group has less than two children.
    LessThan2Children,
    /// Strength couldn't be determined because this group has no siblings (ie. its parent has only one child).
    NoSiblings,
    /// Strength couldn't be determined for root groups and this _is_ a root group.
    Root,
    Ok(Strength),
}

pub struct GroupInfo {
    pub strength: StrengthInfo,
    /// If [None], the group has less than two children
    pub separation: Option<Distance>,
}

/// An item that can be sorted using the geogroup algorithm
pub trait SortableItem: Point {
    type Time: Ord + Clone;
    type Position: Point + Clone;

    fn get_time(&self) -> Self::Time;
    fn get_position(&self) -> Self::Position;
}

fn provide_info<Item: SortableItem, InnerNode>(
    tree: BinTree<Item, InnerNode, ()>,
) -> BinTree<Item, InnerNode, NodeInfo<Item>> {
    match tree {
        BinTree::InnerNode(node) => {
            let children = Box::new(node.children.map(|c| provide_info(c)));
            BinTree::InnerNode(BTInnerNode {
                node_data: NodeInfo {
                    first_time: children[0].get_node_data().first_time.clone(),
                    last_time: children[children.len() - 1]
                        .get_node_data()
                        .last_time
                        .clone(),
                    first_pos: children[0].get_node_data().first_pos.clone(),
                    last_pos: children[children.len() - 1]
                        .get_node_data()
                        .last_pos
                        .clone(),
                },
                children,
                inner_node_data: node.inner_node_data,
            })
        }
        BinTree::Leaf(l, ()) => {
            let tree_group_info = NodeInfo {
                first_time: l.get_time(),
                last_time: l.get_time(),
                first_pos: l.get_position(),
                last_pos: l.get_position(),
            };
            BinTree::Leaf(l, tree_group_info)
        }
    }
}

impl<Item: SortableItem> DeepSorter<Item> {
    pub fn new(points: Vec<Item>, params: Params) -> Self {
        //points.sort_unstable_by_key(|it| it.get_time());

        Self {
            bintree: provide_info(sort_to_binary_tree(points)),
            params,
        }
    }

    //pub fn add_items(&mut self, item: impl IntoIterator<Item = Item>) { todo!() }

    /// Get the smallest group which still encapsulates a given time
    fn get_time_group_mut(
        &mut self,
        time: &Item::Time,
    ) -> Option<&mut BTInnerNode<Item, (), NodeInfo<Item>>> {
        get_time_group_mut(&mut self.bintree, time)
    }
}

/// Check if a [`BinTree`] contains the given time.
///
/// Same behavior as `get_time_group_mut.is_some()`
///
/// Returns false if the `tree` itself is out of range, that is if there are either
/// no newer items than `time` OR if there are no older items than `time`.
/// This means that for `BinTree::Leaf` we always return [false].
fn tree_contains_time<Item: SortableItem>(
    tree: &BinTree<Item, (), NodeInfo<Item>>,
    time: &Item::Time,
) -> bool {
    match tree {
        BinTree::Leaf(_, _) => false,
        BinTree::InnerNode(node) => {
            !(node.node_data.first_time > *time || node.node_data.last_time < *time)
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
fn get_time_group_mut<'a, Item: SortableItem>(
    tree: &'a mut BinTree<Item, (), NodeInfo<Item>>,
    time: &Item::Time,
) -> Option<&'a mut BTInnerNode<Item, (), NodeInfo<Item>>> {
    match tree {
        BinTree::Leaf(_, _) => None,
        BinTree::InnerNode(node) => {
            if node.node_data.first_time > *time || node.node_data.last_time < *time {
                return None;
            }
            // The following code is a bit dirty, because using just recursion into this function and
            // checking for `None` leads to borrow checker crying

            // First check if left child contains a more specific group
            if tree_contains_time(&node.children[0], time) {
                return get_time_group_mut(&mut node.children[0], time);
            }

            // Then check if right child contains a more specific group
            if tree_contains_time(&node.children[1], time) {
                return get_time_group_mut(&mut node.children[1], time);
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
