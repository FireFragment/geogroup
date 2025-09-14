use either::Either;

use super::*;

/// See [`dissolve_by_key`](GroupRefUtils::dissolve_by_key)
#[derive(Clone)]
pub struct Dissolver<G: GroupRef, F: Fn(&G) -> bool + Clone> {
    this: G,
    /// Should never change its output for same inputs
    fun_should_dissolve: F,
    /// This is true for all [`DissolverGroupRef`]s created by API users.
    ///
    /// It's false only for children created in [`DissolverGroupRef::get_children`].
    /// This is used to ensure that created children are never marked by `fun` to be dissolved.
    /// However, this is OK (albeit a bit weird) if [`DissolverGroupRef`] was created by an API user.
    #[cfg(debug_assertions)]
    is_root: bool,
}

impl<G: GroupRef, F: Fn(&G) -> bool + Clone> Dissolver<G, F> {
    pub fn new(this: G, fun_should_dissolve: F) -> Self {
        Self {
            this,
            fun_should_dissolve,

            #[cfg(debug_assertions)]
            is_root: true,
        }
    }
}

impl<OrigGr: GroupRef, F: Fn(&OrigGr) -> bool + Clone> GroupRef for Dissolver<OrigGr, F> {
    type NodeData = OrigGr::NodeData;

    type LeafData = OrigGr::LeafData;

    type GroupData = OrigGr::GroupData;

    type StructureErr = OrigGr::StructureErr;

    type LeafRef = OrigGr::LeafRef;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        #[cfg(debug_assertions)]
        if !self.is_root {
            debug_assert!(
                !(self.fun_should_dissolve)(&self.this),
                "There's a DissolverGroupRef which should have been dissolved"
            );
        }

        Ok(
            get_children_dissolved(&self.this, &self.fun_should_dissolve.clone())?
                .into_iter(),
        )
    }

    fn group_data(&self) -> Self::GroupData {
        self.this.group_data()
    }

    fn node_data(&self) -> Self::NodeData {
        self.this.node_data()
    }
}

fn get_children_dissolved<'a, OrigGr: GroupRef + 'a, F: Fn(&OrigGr) -> bool + Clone>(
    group: &OrigGr,
    fun_should_dissolve: &'a F,
) -> Result<Vec<NodeRef<Dissolver<OrigGr, F>>>, <OrigGr as GroupRef>::StructureErr> {
    Ok(group
        .get_children()?
        .map(|child| {
            Ok(match child {
                NodeRef::Leaf(leaf) => Either::Left(std::iter::once(NodeRef::Leaf(leaf))),
                NodeRef::Group(subgroup) => {
                    if (fun_should_dissolve)(&subgroup) {
                        Either::Right(
                            get_children_dissolved(&subgroup, fun_should_dissolve)?.into_iter(),
                        )
                    } else {
                        Either::Left(std::iter::once(NodeRef::Group(Dissolver {
                            this: subgroup,
                            fun_should_dissolve: fun_should_dissolve.clone(),

                            #[cfg(debug_assertions)]
                            is_root: false,
                        })))
                    }
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}
