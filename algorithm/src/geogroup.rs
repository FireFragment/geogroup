use core::fmt;
use std::{hash::Hash, rc::Rc, sync::Arc};
use lazy_hierarchy::{GroupRef, GroupRefUtils};
use std::fmt::Debug;

pub use deep_sorter::{GroupInfo};

use super::*;

pub struct NodeInfo<NDItem = (), NameErr = Infallible> {
    /// If you join all `local_id_path`s of a node's parents, you get the unique _identification path_ of the node.
    /// This _identification path_ is preserved during algorithm parameter changes, so it can be used to track selection,
    /// animating the nodes etc.
    pub local_id_path: Vec<HorizontalIdx>,
    /// [`None`] if it wan't named yet, [`Err`] if naming resulted in an error
    pub name: Option<Result<Vec<NDItem>, NameErr>>
}

pub struct Sorter<Item: SortableItem, NDItem: Clone + PartialEq + Eq + Hash = (), NameErr = Infallible> {
    deep_sorter: Arc<DeepSorter<Item, NDItem, NameErr>>,
    params: Params,
}

impl<Item: SortableItem + Debug, NDItem: Clone + PartialEq + Eq + Hash + Debug, NameErr: fmt::Debug> Debug for Sorter<Item, NDItem, NameErr>
where
    Item::Time: std::fmt::Debug,
    Item::Position: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sorter")
            .field("deep_sorter", &self.deep_sorter)
            .field("params", &self.params)
            .finish()
    }
}

impl<Item: SortableItem + Debug, NDItem: Clone + PartialEq + Eq + Hash> Sorter<Item, NDItem> {
    /// Pretty print the resulting tree and parameters
    pub fn debug(&self) {
        self.debug_with_fmt_leafs(|l| format!("{:?}",  l));
    }

    /// Pretty print the resulting tree and parameters
    pub fn debug_with_fmt_leafs(&self, fmt_leaf: impl Fn(&Item) -> String) {
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
                            "Separation: {:?}, Strength: .{:0>2.0}, Ratio: {separation_ratio}",
                            g.group_data().separation,
                            strength as f32 * 100.0 / Strength::MAX as f32
                        ),
                        issue => format!(
                            "Separation: {:?}, Strength: {issue:?}",
                            g.group_data().separation,
                        ),
                    },
                    lazy_hierarchy::NodeRef::Leaf(l) => fmt_leaf(l.leaf_data()),
                },
                true
            )
        );
    }
}

impl<Item: SortableItem, NDItem: Clone + PartialEq + Eq + Hash, NameErr: Clone> Sorter<Item, NDItem, NameErr> {
    pub fn hierarchy<'s>(
        &'s self,
    ) -> impl lazy_hierarchy::GroupRef<
        GroupData = GroupInfo,
        LeafData = &'s Item,
        NodeData = NodeInfo<NDItem, NameErr>,
        StructureErr = Infallible,
    > {
        /// Data that is passed from dissolved groups to their children
        #[derive(Clone, Debug)]
        struct InheritedData<NDItem> {
            path_part: Vec<HorizontalIdx>,
            naming_data: Vec<NDItem>
        }

        impl<NDItem> Default for InheritedData<NDItem> {
            fn default() -> Self {
                Self { path_part: Default::default(), naming_data: Default::default() }
            }
        }

        self.deep_sorter()
            .deep_hierarchy()
            .with_parent()
            .dissolve_by_key_and_fold(|group| {
                let create_inherited_data = || InheritedData {
                    path_part: vec![group.node_data().data.id],
                    naming_data: group.node_data().data.get_naming_data()
                        .map(|res| res.as_ref().ok().cloned())
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| Vec::new()),
                };

                match group.group_data().strength {
                    StrengthInfo::Ok{
                        strength,
                        ..
                    } => {
                        // MINIMUM DISTANCE
                        // For this, we use parents separation, not our own, because if this group's spearation
                        // is low but our paerents spearation is high, we don't want to dissolve this
                        // low-separation group into the high separation group and spam it - in case of low-separation
                        // groups, we want to only dissolve its children, so we use parent spearation when deciding whether
                        // to dissolve
                        if group.node_data().parent
                                .map(|parent| parent.group_data().separation)
                                .flatten()
                                .unwrap_or(Distance::MAX)
                            < self.params.minimum_distance
                            || strength < MAX_DEPTH - self.params.depth
                        {
                            Some(create_inherited_data())
                        } else { None }
                    },
                    StrengthInfo::Root => None,

                    // These shouldn't happen, but we can somehow (albeit non-perfectly) handle them anyway
                    StrengthInfo::LessThan2ChildrenWithLocation => {
                        log::error!("Found a node with less than two children in `deep_sorter`. Recovery is easy, but this shouldn't happen.");
                        Some(create_inherited_data())
                    }
                    StrengthInfo::NoSiblings => {
                        log::error!("Found a node with no siblings in `deep_sorter`. Recovery is easy, but this shouldn't happen.");
                        None
                    },
                }},
                |mut data1, data2| {
                    data1.path_part.extend_from_slice(&data2.path_part);
                    data1.naming_data.extend_from_slice(&data2.naming_data);
                    data1
                }
            )
            // Reverse with_parent call above
            .map_node_data(|node| {
                let node_data = node.node_data();
                let mut id_path = node_data.inherited.path_part;
                id_path.push(node_data.original.data.id.clone());
                let name = node_data.original.data.get_naming_data().map(|n|
                    n.as_ref()
                        .map(|name| [name.clone(), node_data.inherited.naming_data].concat())
                        .map_err(|e| e.clone())
                ).ok();
                NodeInfo { local_id_path: id_path, name }
            })
    }
}

/// # Simple getters and setters
impl<Item: SortableItem, NDItem: Clone + PartialEq + Eq + Hash, NameErr> Sorter<Item, NDItem, NameErr> {
    /// This sorts all items to binary tree, potentially long-running
    pub fn new(items: Vec<Item>, params: Params) -> Self {
        Self {
            deep_sorter: Arc::new(DeepSorter::new(items)),
            params,
        }
    }

    pub fn deep_sorter(&self) -> &DeepSorter<Item, NDItem, NameErr> {
        &*self.deep_sorter
    }

    pub fn deep_sorter_rc(&self) -> Arc<DeepSorter<Item, NDItem, NameErr>> {
        self.deep_sorter.clone()
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut Params {
        &mut self.params
    }
}
