use either::Either;

use super::*;

/// See [`dissolve_by_key`](GroupRefUtils::dissolve_by_key)
#[derive(Clone)]
pub struct Dissolver<
    G: GroupRef,
    InheritedData: Clone,

    FnDissolve: Fn(&G) -> Option<InheritedData> + Clone,
    FnFold: Fn(InheritedData, InheritedData) -> InheritedData + Clone,
>
{
    this: G,
    /// Returns [Some] if the group should be dissolved, [None] otherwise.
    /// The data it returns are then passed to its children
    /// Should never change its output for same inputs
    fun_dissolve: FnDissolve,
    /// Function to merge T values when multiple levels are dissolved
    fun_fold: FnFold,

    /// T data from dissolved ancestors that should be merged into this group
    ///
    /// Default value if the ancestor has not been dissolved
    inherited_data: InheritedData,
    /// This is true for all [`DissolverGroupRef`]s created by API users.
    ///
    /// It's false only for children created in [`DissolverGroupRef::get_children`].
    /// This is used to ensure that created children are never marked by `fun` to be dissolved.
    /// However, this is OK (albeit a bit weird) if [`DissolverGroupRef`] was created by an API user.
    #[cfg(debug_assertions)]
    is_root: bool,
}

pub fn new<
    'a,
    G: GroupRef,
    F: Fn(&G) -> bool + Clone + 'a
>(this: G, fun_should_dissolve: F) -> Dissolver<
    G,
    (),
    impl Fn(&G) -> Option<()> + Clone + 'a,
    impl Fn((), ()) -> () + Clone + Send + Sync + 'static
>  {
    Dissolver {
        this,
        fun_dissolve: move |g| if fun_should_dissolve(g) { Some(()) } else { None },
        fun_fold: |_, _| (),
        inherited_data: (),

        #[cfg(debug_assertions)]
        is_root: true,
    }
}

pub fn new_folding<
    G: GroupRef,
    InheritedData: Clone + Default,

    FnDissolve: Fn(&G) -> Option<InheritedData> + Clone,
    FnFold: Fn(InheritedData, InheritedData) -> InheritedData + Clone,
>(this: G, fun_dissolve: FnDissolve, fun_fold: FnFold) -> Dissolver<G, InheritedData, FnDissolve, FnFold>  {
    Dissolver {
        this,
        fun_dissolve,
        fun_fold,
        inherited_data: Default::default(),

        #[cfg(debug_assertions)]
        is_root: true,
    }
}

/// [`NodeData`](GroupRef::NodeData) of [Dissolver]
pub struct NodeData<Orig, Inherited> {
    /// The original node data
    pub original: Orig,
    /// The data inherited from parent
    pub inherited: Inherited,
}

impl<
    OrigGr: GroupRef,
    InheritedData: Clone,

    FnDissolve: Fn(&OrigGr) -> Option<InheritedData> + Clone,
    FnFold: Fn(InheritedData, InheritedData) -> InheritedData + Clone,
> GroupRef for Dissolver<OrigGr, InheritedData, FnDissolve, FnFold>
{
    type NodeData = NodeData<OrigGr::NodeData, InheritedData>;
    type LeafData = OrigGr::LeafData;
    type GroupData = OrigGr::GroupData;
    type StructureErr = OrigGr::StructureErr;
    type LeafRef = DissolverLeaf<OrigGr::LeafRef, InheritedData>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        #[cfg(debug_assertions)]
        if !self.is_root {
            debug_assert!(
                !((self.fun_dissolve)(&self.this).is_some()),
                "There's a DissolverGroupRef which should have been dissolved"
            );
        }

        Ok(
            get_children_dissolved(
                &self.this,
                &self.fun_dissolve,
                &self.fun_fold,
                self.inherited_data.clone()
            )?
            .into_iter()
        )
    }

    fn group_data(&self) -> Self::GroupData {
        self.this.group_data()
    }

    fn node_data(&self) -> Self::NodeData {
        NodeData {
            original: self.this.node_data(),
            inherited: self.inherited_data.clone() // TODO: Can we get away without cloning it?
        }
    }
}

#[derive(Clone)]
pub struct DissolverLeaf<OrigLeaf: LeafRef, InheritedData: Clone> {
    orig: OrigLeaf,
    inherited_data: InheritedData,
}

impl<OrigLeaf: LeafRef, InheritedData: Clone> LeafRef for DissolverLeaf<OrigLeaf, InheritedData> {
    type LeafData = OrigLeaf::LeafData;
    type NodeData = NodeData<OrigLeaf::NodeData, InheritedData>;

    fn leaf_data(&self) -> Self::LeafData {
        self.orig.leaf_data()
    }

    fn node_data(&self) -> Self::NodeData {
        NodeData {
            original: self.orig.node_data(),
            inherited: self.inherited_data.clone(), // TODO: Can we get away without cloning it? eg. Rc
        }
    }
}

// The recursive part of dissolver (recurses down on dissolved groups)
fn get_children_dissolved<
    'a,
    OrigGr: GroupRef,
    InheritedData: Clone,

    FnDissolve: Fn(&OrigGr) -> Option<InheritedData> + Clone,
    FnFold: Fn(InheritedData, InheritedData) -> InheritedData + Clone,
>(
    parent_group: &OrigGr,
    fun_dissolve: &'a FnDissolve,
    fun_fold: &'a FnFold,
    parent_inherited_data: InheritedData,
) -> Result<Vec<NodeRef<Dissolver<OrigGr, InheritedData, FnDissolve, FnFold>>>, <OrigGr as GroupRef>::StructureErr>
{
    Ok(parent_group
        .get_children()?
        .map(move |child| {
            Ok(match child {
                NodeRef::Leaf(leaf) => Either::Left(std::iter::once(NodeRef::Leaf(DissolverLeaf {
                    orig: leaf,
                    inherited_data: parent_inherited_data.clone(),
                }))),
                NodeRef::Group(subgroup) => {
                    if let Some(subgroup_inherited_data) = (fun_dissolve)(&subgroup) {
                        // If `subgroup` should be dissolved
                        let new_inherited_data = (fun_fold)(parent_inherited_data.clone(), subgroup_inherited_data);
                        Either::Right(
                            get_children_dissolved(&subgroup, fun_dissolve, fun_fold, new_inherited_data)?.into_iter(),
                        )
                    } else {
                        // If `subgroup` should not be dissolved, we map to a single element iterator (`iter::once`)
                        // so that even after call to `flatten` below, it survives.
                        Either::Left(std::iter::once(NodeRef::Group(Dissolver {
                            this: subgroup,
                            fun_dissolve: fun_dissolve.clone(),
                            fun_fold: fun_fold.clone(),
                            inherited_data: parent_inherited_data.clone(),

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
