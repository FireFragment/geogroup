use futures::{FutureExt, StreamExt};
use std::hash::Hash;
use std::{collections::HashSet, iter};

use super::*;

// TODO: Extract to naming
/// `fun_name_node` should use interior mutability.
/// Please note, that this repeatedly locks and unlocks the locks in the node, so
/// states of the tree where some nodes are named and some are not may be obesrved.
/// Therefore, try to make sure this function is not run multiple times at once (concurrently)
///
/// Returns the _full_ name of the node. Sets the name of children of the node, but not of the node itself.
///
/// `fun_get_leaf_name` - generate the name of the node
/// `fun_name_node` - sets name of the node. If it has already been named, the behavior could be arbitrary
/// (either that it sets the value again or ignores it)
/// `fun_node_name` - get name of the node if it has already been named
pub async fn try_naming_generic<
    'a,
    G: lazy_hierarchy::GroupRef<StructureErr = impl fmt::Debug> + 'a,
    NDItem: PartialEq + Eq + Hash + Clone,
>(
    node: lazy_hierarchy::NodeRef<G>,
    fun_get_leaf_name: &impl AsyncFn(&G::LeafRef) -> HashSet<NDItem>,
    fun_name_node: &impl AsyncFn(&lazy_hierarchy::NodeRef<G>, &HashSet<NDItem>),
    fun_node_name: &impl AsyncFn(&lazy_hierarchy::NodeRef<G>) -> Option<HashSet<NDItem>>,
) -> HashSet<NDItem> {
    fun_node_name(&node)
        .await
        .map(|it| Either::Left(async { it }))
        .unwrap_or(Either::Right(async {
            match &node {
                NodeRef::Group(group) => {
                    let full_names = futures::future::join_all(
                        group.get_children().expect("TODO").map(|child| {
                            //HashSet::<NDItem>::new()
                            Box::pin(try_naming_generic(
                                child,
                                fun_get_leaf_name,
                                fun_name_node,
                                fun_node_name,
                            ))
                        }),
                    ).await;

                    let name_of_this = full_names
                        .clone()
                        .into_iter()
                        .reduce(|acc_name, name_to_add| {
                            if acc_name.is_empty() {
                                name_to_add
                            } else {
                                acc_name
                                    .into_iter()
                                    .filter(|acc_name_item| name_to_add.contains(acc_name_item))
                                    .collect()
                            }
                        }).unwrap_or_default(); // TODO: Is this the correct default?

                    for (child, mut child_name) in group.get_children().expect("TODO").zip(full_names) {
                        for nditem_to_remove in &name_of_this {
                            child_name.remove(nditem_to_remove);
                        }
                        fun_name_node(&child, &child_name).await;
                    }

                    name_of_this
                }
                NodeRef::Leaf(leaf) => fun_get_leaf_name(&leaf).await,
            }
        }))
        .await
}

impl<Item: SortableItem, NDItem: Clone + PartialEq + Eq + Hash> DeepSorter<Item, NDItem> {
    pub async fn try_naming(&self, fun_get_leaf_name: impl AsyncFn(&Item) -> HashSet<NDItem>) {
        // TODO: Do some locks so that it can't be launched multiple times simoultaneously
        try_naming_generic(
            lazy_hierarchy::NodeRef::Group(self.deep_hierarchy()),
            &async |leaf| fun_get_leaf_name(leaf.leaf_data()).await,
            &async |node, name| {
                let _ = node
                    .node_data()
                    .static_info
                    .naming_data
                    .set(name.into_iter().cloned().collect());
            },
            &async |node| {
                node.node_data()
                    .static_info
                    .naming_data
                    .get()
                    .map(|n| n.into_iter().cloned().collect())
            },
        )
        .await;
    }
}
