use itertools::PeekingNext as _;

use super::*;

/// id_path of root is ignored
///
/// This is general implementation, you are likely interested in
/// [the specific implementation for main hierarchy](main_hierarchy::TemplateHiearchy::selection_to_indices)
pub fn indicies_to_selection<
    H: lazy_hierarchy::GroupRef,
    I: Iterator<Item = u8>,
    F: Fn(&lazy_hierarchy::NodeRef<H>) -> Option<I>,
>(
    hierarchy: H,
    get_local_id_path: F,
    mut indices_iter: impl Iterator<Item = usize>,
) -> impl Iterator<Item = PathComponent> {
    let mut current_node = lazy_hierarchy::NodeRef::Group(hierarchy);

    iter::from_fn(move || {
        let Some(idx) = indices_iter.next() else {
            return None;
        };
        let child = {
            let lazy_hierarchy::NodeRef::Group(ref current_group) = current_node else {
                return None;
            };
            let Ok(mut children) = current_group.get_children() else {
                return None;
            };
            let Some(child) = children.nth(idx) else {
                return None;
            };
            child
        };

        match get_local_id_path(&child) {
            Some(loc_id_path) => {
                current_node = child;
                Some(Either::Left(
                    loc_id_path.map(|id| PathComponent::IdPath(id)),
                ))
            }
            None => Some(Either::Right(iter::once(PathComponent::Index(idx)))),
        }
    })
    .flatten()
}

/// Convert selection in form of [`PathComponent`]s to simple indices
///
/// This is general implementation, you are likely interested in
/// [the specific implementation for main hierarchy](main_hierarchy::TemplateHiearchy::selection_to_indices)
///
/// The iterator `selection_iter` should be fast to clone
pub fn selection_to_indicies<
    H: lazy_hierarchy::GroupRef,
    F: Fn(&lazy_hierarchy::NodeRef<H>) -> Option<LocI>,
    LocI: Iterator<Item = u8>,
    I: Iterator<Item = PathComponent> + Clone,
>(
    hierarchy: H,
    get_local_id_path: F,
    selection_iter: I,
) -> impl Iterator<Item = usize> + use<H, F, I, LocI> {
    let mut current_node = lazy_hierarchy::NodeRef::Group(hierarchy);
    let mut selection_iter = selection_iter.peekable();

    //if let Some(mut current_component) = selection_iter.next() {
    iter::from_fn(move || {
        let lazy_hierarchy::NodeRef::Group(ref current_group) = current_node else {
            return None;
        };
        let Ok(children) = current_group.get_children() else {
            return None;
        };

        for (idx, node) in children.collect_vec().into_iter().enumerate() {
            // If the node has ID path...
            if let Some(local_id_path_iter) = get_local_id_path(&node) {
                let mut local_id_path_iter = local_id_path_iter.peekable();
                // Clone of `selection_iter` so that we advance it only when it matches.
                let mut temp_selection_iter = selection_iter.clone();

                // ...consume `local_selection_iter` until its components become different from `local_id_path`...
                temp_selection_iter
                    .peeking_take_while(|selection_component| {
                        local_id_path_iter
                            .peeking_next(|loc_id_path_component| {
                                *selection_component
                                    == PathComponent::IdPath(*loc_id_path_component)
                            })
                            .is_some()
                    })
                    .for_each(|_| ()); // Exhaust the iterator to actually run its code

                // ...then check if `loc_id_path_iter` has been entirely exhausted by the code above.
                // If it was, then it means that it matches with selection perfectly and the `node` is selected
                if local_id_path_iter.next().is_none() {
                    // We also "advance" the selection_iter so that its children use the deeper parts of the selection path
                    selection_iter = temp_selection_iter;
                    current_node = node;
                    return Some(idx);
                }
            }
            // If the node has no ID path, we try to find the selection by its index
            else {
                if selection_iter.peek() == Some(&&PathComponent::Index(idx)) {
                    selection_iter.next(); // We advance the selection_iter if we found a match so that its children use the deeper parts of the selection path
                    current_node = node;
                    return Some(idx);
                }
            }
        }

        return None;
    })
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PathComponent {
    /// An index in the parent
    Index(usize),
    /// An element from an [ID path](backend::main_hierarchy::NodeData::local_id_path)
    IdPath(u8),
}

impl main_hierarchy::TemplateHiearchy {
    /// Convert selection in form of [`PathComponent`]s to simple indices
    ///
    /// The iterator `selection_iter` should be fast to clone
    pub fn selection_to_indices<I: Iterator<Item = PathComponent> + Clone>(
        &self,
        selection_iter: I,
    ) -> impl Iterator<Item = usize> + use<'_, I> {
        selection_to_indicies(
            self.root_final(),
            |node| node.node_data().local_id_path.map(|it| it.into_iter()),
            selection_iter,
        )
    }

    pub fn selection_to_node_data(
        &self,
        selection_iter: impl Iterator<Item = PathComponent> + Clone,
    ) -> main_hierarchy::NodeData {
        let indices = self.selection_to_indices(selection_iter);
        let mut current_group = self.root_final();

        for idx in indices {
            let Ok(mut children) = current_group.get_children() else {
                return current_group.node_data();
            };

            let Some(child) = children.nth(idx) else {
                return current_group.node_data();
            };
            drop(children);

            match child {
                lazy_hierarchy::NodeRef::Group(g) => current_group = g,
                lazy_hierarchy::NodeRef::Leaf(l) => return l.node_data(),
            }
        }

        current_group.node_data()
    }

    pub fn selection_to_static_id(
        &self,
        selection_iter: impl Iterator<Item = PathComponent> + Clone,
    ) -> Option<u64> {
        self.selection_to_node_data(selection_iter).static_id
    }

    /// Convert simple indices to selection in form of [`PathComponent`]s
    pub fn indices_to_selection<I: Iterator<Item = usize>>(
        &self,
        indices_iter: I,
    ) -> impl Iterator<Item = PathComponent> + use<'_, I> {
        indicies_to_selection(
            self.root_final(),
            |node| node.node_data().local_id_path.map(|it| it.into_iter()),
            indices_iter,
        )
    }
}
