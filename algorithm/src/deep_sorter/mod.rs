mod onetime;
//#[cfg(test)]
//mod test;

use std::{convert::Infallible, ops::Div};

use crate::*;
use itertools::Itertools;
pub use onetime::*;

pub struct DeepSorter<Item: SortableItem> {
    bintree: BinTree<Item, (), StaticNodeInfo<Item>>,
}

impl<Item: SortableItem + std::fmt::Debug> std::fmt::Debug for DeepSorter<Item>
where
    Item::Time: std::fmt::Debug,
    Item::Position: std::fmt::Debug,
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
impl<Item: SortableItem> DeepSorter<Item> {
    pub fn deep_hierarchy<'s>(
        &'s self,
    ) -> impl lazy_hierarchy::GroupRef<
        GroupData = GroupInfo,
        LeafData = &'s Item,
        NodeData = NodeInfo<Item>,
        StructureErr = Infallible,
    > + 's {
        self.bintree
            .root()
            // TODO: Consider isolating these computations somewhere else, eg. to `geogroup.rs` or `strength.rs`
            // Calculate separation
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
                        StrengthInfo::LessThan2Children
                    },
                }
            })
            .with_idx()
            // "Revert" with_parent and add index to
            .map_node_data(|node_ref| {
                let node_data = node_ref.node_data();
                NodeInfo { static_info: node_data.data.data.to_owned(), id: match node_data.index {
                    0 => HorizontalIdx::Left,
                    1 => HorizontalIdx::Right,
                    idx => {
                        debug_assert!(false, "Index bigger than 1 in binary tree: {idx}");
                        HorizontalIdx::Right
                    }
                } }
            })
        //
    }
}

/// Information about nodes which is generated once and then used instead of being generated on the fly
#[derive(Debug)]
pub struct StaticNodeInfo<Item: SortableItem> {
    first_time: Item::Time,
    last_time: Item::Time,

    first_pos: Item::Position,
    last_pos: Item::Position,
}

pub struct NodeInfo<Item: SortableItem> {
    static_info: StaticNodeInfo<Item>,
    pub id: HorizontalIdx
}

impl<Item: SortableItem> Clone for StaticNodeInfo<Item> {
    fn clone(&self) -> Self {
        Self {
            first_time: self.first_time.clone(),
            last_time: self.last_time.clone(),
            first_pos: self.first_pos.clone(),
            last_pos: self.last_pos.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StrengthInfo {
    /// Strength couldn't be determined because this group has less than two children.
    LessThan2Children,
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
fn provide_info<Item: SortableItem, InnerNode>(
    tree: BinTree<Item, InnerNode, ()>,
) -> BinTree<Item, InnerNode, StaticNodeInfo<Item>> {
    match tree {
        BinTree::InnerNode(node) => {
            let children = Box::new(node.children.map(|c| provide_info(c)));
            BinTree::InnerNode(BTInnerNode {
                node_data: StaticNodeInfo {
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
            let tree_group_info = StaticNodeInfo {
                first_time: l.get_time(),
                last_time: l.get_time(),
                first_pos: l.get_position().expect("POSERR"),
                last_pos: l.get_position().expect("POSERR"),
            };
            BinTree::Leaf(l, tree_group_info)
        }
    }
}

impl<Item: SortableItem> DeepSorter<Item> {
    /// This sorts all items to binary tree, potentially long-running
    pub fn new(items: Vec<Item>) -> Self {
        //points.sort_unstable_by_key(|it| it.get_time());

        Self {
            bintree: provide_info(sort_to_binary_tree(items)),
        }
    }

    //pub fn add_items(&mut self, item: impl IntoIterator<Item = Item>) { todo!() }

    /// Get the smallest group which still encapsulates a given time
    fn get_time_group_mut(
        &mut self,
        time: &Item::Time,
    ) -> Option<&mut BTInnerNode<Item, (), StaticNodeInfo<Item>>> {
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
    tree: &BinTree<Item, (), StaticNodeInfo<Item>>,
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
    tree: &'a mut BinTree<Item, (), StaticNodeInfo<Item>>,
    time: &Item::Time,
) -> Option<&'a mut BTInnerNode<Item, (), StaticNodeInfo<Item>>> {
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
