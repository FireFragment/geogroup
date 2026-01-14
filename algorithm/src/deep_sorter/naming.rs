use futures::{FutureExt, StreamExt};
use itertools::Itertools as _;
use std::hash::Hash;
use std::{collections::HashSet, iter};

use super::*;

#[derive(thiserror::Error, Debug, Clone, Hash)]
pub enum NamingErr<LeafNamingError> {
    /// Couldn't name group, because no child has successful name
    #[error("All descendants of the group failed to be named")]
    ChildrenFailed,
    /// This is a leaf and the naming failed
    #[error(transparent)]
    Leaf(#[from] LeafNamingError),
}

/// Return value of [`try_naming_generic`]
#[derive(Debug, Clone)]
pub enum NamingRes<NDItem, LeafNameErr> {
    /// We don't return the error if it's already named, because that would require cloning it which may not be possible
    AlreadyNamed(Option<HashSet<NDItem>>),
    NewlyNamed(Result<HashSet<NDItem>, NamingErr<LeafNameErr>>),
}

impl<NDItem, LeafNameErr> NamingRes<NDItem, LeafNameErr> {
    pub fn into_option(self) -> Option<HashSet<NDItem>> {
        match self {
            NamingRes::AlreadyNamed(inner) => inner,
            NamingRes::NewlyNamed(inner) => inner.ok(),
        }
    }

    pub fn as_option(&self) -> Option<&HashSet<NDItem>> {
        match self {
            NamingRes::AlreadyNamed(inner) => inner.as_ref(),
            NamingRes::NewlyNamed(inner) => inner.as_ref().ok(),
        }
    }
}

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
///     - The outer option means whether the name has been set (ie. if this returns [`None`], it means that it has not been set)
///     - The inner option is [`None`] iff there was attempt to name the file, but it failed (ie. if this returns `Some(None)`, it means that we attempted to name the file, but it failed)
pub async fn try_naming_generic<
    'a,
    G: lazy_hierarchy::GroupRef<StructureErr = impl fmt::Debug> + 'a,
    NDItem: PartialEq + Eq + Hash + Clone,
    LeafNameErr,
>(
    node: lazy_hierarchy::NodeRef<G>,
    fun_get_leaf_name: &impl AsyncFn(&G::LeafRef) -> Result<HashSet<NDItem>, LeafNameErr>,
    fun_name_node: &impl AsyncFn(
        &lazy_hierarchy::NodeRef<G>,
        Result<HashSet<NDItem>, NamingErr<LeafNameErr>>,
    ),
    fun_node_name: &impl AsyncFn(&lazy_hierarchy::NodeRef<G>) -> Option<Option<HashSet<NDItem>>>,
) -> NamingRes<NDItem, LeafNameErr> {
    fun_node_name(&node)
        .await
        .map(|it| Either::Left(async { NamingRes::AlreadyNamed(it) }))
        .unwrap_or(Either::Right(async {
            NamingRes::NewlyNamed(match &node {
                NodeRef::Group(group) => {
                    let new_full_names = futures::future::join_all(
                        group.get_children().expect("TODO").map(|child| {
                            //HashSet::<NDItem>::new()
                            Box::pin(try_naming_generic(
                                child,
                                fun_get_leaf_name,
                                fun_name_node,
                                fun_node_name,
                            ))
                        }),
                    )
                    .await;

                    // Is `None` iff no children were named successfully
                    let full_name_of_this = new_full_names
                        .iter()
                        .map(|it| it.as_option())
                        .flatten() // Ignore error names when naming parent
                        .cloned()
                        .reduce(|acc_name, name_to_add| {
                            if acc_name.is_empty() {
                                name_to_add
                            } else {
                                acc_name
                                    .into_iter()
                                    .filter(|acc_name_item| name_to_add.contains(acc_name_item))
                                    .collect()
                            }
                        });

                    //match name_of_this {}

                    // Write the actual naming data to children
                    for (child, child_full_name) in
                        group.get_children().expect("TODO").zip(new_full_names)
                    {
                        let child_full_name = match child_full_name {
                            // If it's already named, we don't need to write the data to it again
                            NamingRes::AlreadyNamed(_) => continue,
                            NamingRes::NewlyNamed(data) => data,
                        };
                        let child_name = child_full_name.map(|mut child_full_name| {
                            // This unwrap suceeds: full_name_of_this could only be `None` when no children were
                            // named successfully and in such case, this part of code doesn't run at all
                            for nditem_to_remove in full_name_of_this.as_ref().unwrap() {
                                child_full_name.remove(nditem_to_remove);
                            }
                            child_full_name
                        });
                        fun_name_node(&child, child_name).await;
                    }

                    match full_name_of_this {
                        Some(name) => Ok(name),
                        None => Err(NamingErr::ChildrenFailed),
                    }
                }
                NodeRef::Leaf(leaf) => fun_get_leaf_name(&leaf).await.map_err(|e| e.into()),
            })
        }))
        .await
}

impl<Item: SortableItem, NDItem: Clone + PartialEq + Eq + Hash, LeafNameErr: Clone>
    DeepSorter<Item, NDItem, NamingErr<LeafNameErr>>
{
    /// Argument `leaf_err_missing_position` is `LeafNameErr` value that should be used in case the item doesn't have known position
    pub async fn try_naming(
        &self,
        //leaf_err_missing_position: impl Fn() -> LeafNameErr,
        fun_get_leaf_name: impl AsyncFn(&Item) -> Result<HashSet<NDItem>, LeafNameErr>,
    ) {
        // TODO: Do some locks so that it can't be launched multiple times simoultaneously
        let root_name = try_naming_generic(
            lazy_hierarchy::NodeRef::Group(
                self.deep_hierarchy()
                    .filter(|node| node.node_data().static_info.is_some()),
            ),
            &async |leaf| fun_get_leaf_name(leaf.leaf_data()).await,
            &async |node, name| {
                let _ = node
                    .node_data()
                    .static_info
                    .unwrap() // We know this succeeds thanks to the `filter` call above
                    .naming_data
                    .set(name.map(|n| n.into_iter().collect()));
            },
            &async |node| {
                node.node_data()
                    .static_info
                    .unwrap() // We know this succeeds thanks to the `filter` call above
                    .naming_data
                    .get()
                    .map(|n| n.as_ref().ok().map(|n| n.iter().cloned().collect()))
            },
        )
        .await;

        match root_name {
            NamingRes::AlreadyNamed(_) => {}
            NamingRes::NewlyNamed(hash_set) => {
                let res = self
                    .deep_hierarchy()
                    .node_data()
                    .static_info
                    .unwrap() // We know this succeeds thanks to the `filter` call above
                    .naming_data
                    .set(hash_set.map(|n| n.into_iter().collect()));
                debug_assert!(res.is_ok()) // The cell should have been unitialized, because `AlreadyNamed` branch would be taken otherwise
            }
        }
    }
}
