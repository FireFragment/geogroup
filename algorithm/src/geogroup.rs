use lazy_hierarchy::{GroupRef, GroupRefUtils};
use std::fmt::Debug;

pub use deep_sorter::{GroupInfo, NodeInfo};

use super::*;

pub struct Sorter<Item: SortableItem> {
    deep_sorter: DeepSorter<Item>,
    params: Params,
}

impl<Item: SortableItem + Debug> Sorter<Item> {
    /// Pretty print the resulting tree and parameters
    pub fn debug(&self) {
        println!(
            "Min. strength: {:.2}",
            self.params().depth as f32 / MAX_DEPTH as f32
        );

        println!(
            "{}",
            self.hierarchy().format_as_tree(
                |n| match n {
                    lazy_hierarchy::NodeRef::Group(g) => match g.group_data().strength {
                        StrengthInfo::Ok {
                            strength,
                            separation_ratio,
                        } => format!(
                            "Strength: .{:0>2.0}, Ratio: {separation_ratio}",
                            strength as f32 * 100.0 / Strength::MAX as f32
                        ),
                        issue => format!("{issue:?}"),
                    },
                    lazy_hierarchy::NodeRef::Leaf(l) => format!("{:?}", l.leaf_data()),
                },
                true
            )
        );
    }
}

impl<Item: SortableItem> Sorter<Item> {
    pub fn hierarchy<'s>(
        &'s self,
    ) -> impl lazy_hierarchy::GroupRef<
        GroupData = GroupInfo,
        LeafData = &'s Item,
        NodeData = NodeInfo<Item>,
        StructureErr = Infallible,
    > {
        self.deep_sorter()
            .deep_hierarchy()
            .dissolve_by_key(|group| match group.group_data().strength {
                StrengthInfo::Ok{strength, ..} => (strength) < MAX_DEPTH - self.params.depth,
                StrengthInfo::Root => false,

                // These shouldn't happen, but we can somehow (albeit non-perfectly) handle them anyway
                StrengthInfo::LessThan2Children => {
                    log::error!("Found a node with less than two children in `deep_sorter`. Recovery is easy, but this shouldn't happen.");
                    true
                }
                StrengthInfo::NoSiblings => {
                    log::error!("Found a node with no siblings in `deep_sorter`. Recovery is easy, but this shouldn't happen.");
                    false
                },
            })
    }
}

/// # Simple getters and setters
impl<Item: SortableItem> Sorter<Item> {
    /// This sorts all items to binary tree, potentially long-running
    pub fn new(items: Vec<Item>, params: Params) -> Self {
        Self {
            deep_sorter: DeepSorter::new(items),
            params,
        }
    }

    pub fn deep_sorter_mut(&mut self) -> &mut DeepSorter<Item> {
        &mut self.deep_sorter
    }

    pub fn deep_sorter(&self) -> &DeepSorter<Item> {
        &self.deep_sorter
    }

    pub fn params(&self) -> &Params {
        &self.params
    }
}
