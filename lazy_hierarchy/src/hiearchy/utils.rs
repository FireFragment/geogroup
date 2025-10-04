//! Utility types for working with hierarchies

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use super::*;
pub mod dissolve;
pub mod map;
pub mod mark_root;
pub mod with_parent;
pub mod with_idx;
pub use dissolve::Dissolver;
pub use mark_root::{MarkRootGroupRef, MarkedRootGroupData};
pub use with_parent::{WithParentGroupRef, WithParentLeafRef, WithParentNodeData};
pub use with_idx::{WithIdxGroupRef, WithIdxLeafRef};

use either::Either;
#[cfg(feature = "colors")]
use owo_colors::OwoColorize;

/// Automatically implemented for all possible types
pub trait GroupRefUtils: GroupRef {
    /// A hierarchy adapter which lets you "fuse" multiple [`GroupRef`]s into a single hierarchy.
    ///
    /// It takes a hierarchy whose [`LeafData`](GroupRef::LeafData) is [`MainLeafData`].
    ///  - Leaves matching [`MainLeafData::RealLeaf`] are left as leaves
    ///  - Leaves matching [`MainLeafData::Subgroup`] are changed to groups
    ///
    /// **Tip:** It's quite hard to get all the types exactly right for this function to be callable.
    /// If you have problems calling this function (and compiler emits unhelpful type errors), try calling
    /// [`fused::FGroupRef::new`] instead - it won't fix any errors but then thecompiler usuallly
    /// emits more useful errors.
    ///
    /// **Note:** this has nothing in common with [`Iterator::fuse`]
    fn fuse<Subgroup: GroupRef>(self) -> fused::FGroupRef<Self, Subgroup>
    where
        Self: GroupRef<
            GroupData = Subgroup::GroupData,
            NodeData = Subgroup::NodeData,
            LeafData = fused::MainLeafData<Subgroup::LeafData, Subgroup>,
            StructureErr = Subgroup::StructureErr,
        >,
    {
        fused::FGroupRef::new(self)
    }

    /// Convert to concrete hierarchy by instantiating all the items
    fn collect_to_concrete<'c>(
        &'c self,
    ) -> Result<concrete::Group<Self::GroupData, Self::LeafData, Self::NodeData>, Self::StructureErr>
    {
        let group_data = self.group_data();
        let node_data = self.node_data();
        let children = self.get_children()?;
        Ok(concrete::Group::new(
            {
                children
                    .map(|child| {
                        Ok(match child {
                            NodeRef::Group(g) => {
                                concrete::Node::new_group(g.collect_to_concrete()?)
                            }
                            NodeRef::Leaf(l) => concrete::Node::new_leaf(concrete::Leaf::new(
                                l.leaf_data(),
                                l.node_data(),
                            )),
                        })
                    })
                    .collect::<Result<_, _>>()?
            },
            group_data,
            node_data,
        ))
    }

    fn map<M: utils::map::Mapper<Self>>(self, mapper: M) -> utils::map::MappedGroupRef<Self, M> {
        utils::map::map(self, mapper)
    }

    fn map_as_groupref<M: utils::map::Mapper<Self>>(
        self,
        mapper: M,
    ) -> utils::map::MappedGroupRef<Self, M> {
        utils::map::map_as_groupref(self, mapper)
    }

    fn map_group_data<GroupDataNew, F: Fn(&Self) -> GroupDataNew + Clone>(
        self,
        fun: F,
    ) -> impl GroupRef<
        GroupData = GroupDataNew,
        LeafData = Self::LeafData,
        NodeData = Self::NodeData,
        StructureErr = Self::StructureErr,
    > {
        utils::map::map_group_data(self, fun)
    }

    fn map_leaf_data<
        LeafDataNew,
        F: Fn(&Self::LeafRef) -> LeafDataNew + std::clone::Clone,
    >(
        self,
        fun: F,
    ) -> impl GroupRef<
        GroupData = Self::GroupData,
        LeafData = LeafDataNew,
        NodeData = Self::NodeData,
        StructureErr = Self::StructureErr,
    > {
        utils::map::map_leaf_data(self, fun)
    }

    fn map_node_data<
        NodeDataNew,
        F: Fn(NodeRef<Self>) -> NodeDataNew + Clone,
    >(
        self,
        fun: F,
    ) -> impl GroupRef<
        GroupData = Self::GroupData,
        LeafData = Self::LeafData,
        NodeData = NodeDataNew,
        StructureErr = Self::StructureErr,
    > {
        utils::map::map_node_data(self, fun)
    }

    fn map_structure_error<
        StructureErrorNew,
        F: Fn(Self::StructureErr, &Self) -> StructureErrorNew + Clone,
    >(
        self,
        fun: F,
    ) -> impl GroupRef<
        GroupData = Self::GroupData,
        LeafData = Self::LeafData,
        NodeData = Self::NodeData,
        StructureErr = StructureErrorNew,
    > {
        utils::map::map_structure_error(self, fun)
    }

    fn with_parent(self) -> utils::with_parent::WithParentGroupRef<Self> {
        utils::with_parent::with_parent(self)
    }

    /// Move all children from a group to its parent group if `fun_should_dissolve` returns [true] for it.
    /// Discards group data and node data for those "dissolved" groups.
    fn dissolve_by_key<'a>(
        self,
        fun_should_dissolve: impl Fn(&Self) -> bool + Clone + 'a,
    ) -> impl GroupRef<
        GroupData = Self::GroupData,
        LeafData = Self::LeafData,
        NodeData = Self::NodeData,
        StructureErr = Self::StructureErr,
    >
    {
        utils::dissolve::new(self, fun_should_dissolve)
            .map_node_data(|node| {
                // There are no inherited data
                let dissolve::NodeData { original, inherited: () } = node.node_data();
                original
            })
    }

    /// Move all children from a group to its parent group if `fun_should_dissolve` returns [Some] for it.
    /// Unlike [`dissolve_by_key`], this version allows passing custom data (`InheritedData`) from dissolved groups
    /// to their children in their [`NodeData`](GroupRef::NodeData).
    ///
    /// You will likely want to run
    /// [`map_node_data`](Dissolver::map_node_data) on return value of this function to process the inherited data.
    ///
    /// # Parameters
    /// - `fun_dissolve`: Function which determines which groups should be dissolved and generates `InheritedData`
    /// - `fun_fold`: Function which merges multiple `InheritedData`
    fn dissolve_by_key_and_fold<
        InheritedData: Clone + Default,
    >(
        self,
        fun_dissolve: impl Fn(&Self) -> Option<InheritedData> + Clone,
        fun_fold: impl Fn(InheritedData, InheritedData) -> InheritedData + Clone,
    ) -> impl GroupRef<
        GroupData = Self::GroupData,
        LeafData = Self::LeafData,
        NodeData = dissolve::NodeData<Self::NodeData, InheritedData>,
        StructureErr = Self::StructureErr,
    >
    where
        Self::GroupData: Clone,
    {
        utils::dissolve::new_folding(self, fun_dissolve, fun_fold)
    }


    /// Utility for marking the root group in a hierarchy
    ///
    /// This utility transforms a hierarchy's `GroupData` from type `T` to [`MarkedRootGroupData<T>`],
    /// where [`is_root`](MarkedRootGroupData::is_root) is `true` only for the root group
    /// (ie. the very group that was passed to this function).
    /// All child groups will have [`is_root`](MarkedRootGroupData::is_root) set to `false`.
    ///
    /// # Example
    ///
    /// ```
    /// use lazy_hierarchy::{concrete::{Group, Leaf, Node}, prelude::*, utils::MarkedRootGroupData};
    ///
    /// let leaf = Leaf::new("leaf_data", ());
    /// let child_group = Group::new(vec![Node::new_leaf(leaf)], "child", ());
    /// let root_group = Group::new(vec![Node::new_group(child_group)], "root", ());
    ///
    /// let marked = root_group.mark_root();
    ///
    /// // Root group is marked as such
    /// assert_eq!(marked.group_data(), MarkedRootGroupData { is_root: true, data: &"root" });
    ///
    /// // Child groups are not marked as root
    /// let children: Vec<_> = marked.get_children().unwrap().collect();
    /// if let lazy_hierarchy::NodeRef::Group(child) = &children[0] {
    ///     assert_eq!(child.group_data(), MarkedRootGroupData { is_root: false, data: &"child" });
    /// }
    /// ```
    fn mark_root(self) -> utils::MarkRootGroupRef<Self> {
        utils::mark_root::mark_root(self)
    }

    /// Transform a hierarchy to include index information in NodeData
    ///
    /// This utility transforms a hierarchy's [`NodeData`](GroupRef::NodeData) from type `T` to [`with_idx::NodeData<T>`],
    /// where each node contains its original data plus its index within its parent group.
    /// The root node will have index `0`.
    ///
    /// You will likely want to run
    /// [`map_node_data`](GroupRefUtils::map_node_data) on return value of this function to process the inherited data.
    ///
    /// # Example
    ///
    /// ```
    /// use lazy_hierarchy::{utils, concrete::{Group, Leaf, Node}, prelude::*};
    ///
    /// let leaf1 = Leaf::new("leaf1", "node_data1");
    /// let leaf2 = Leaf::new("leaf2", "node_data2");
    /// let child_group = Group::new(vec![Node::new_leaf(leaf1), Node::new_leaf(leaf2)], "child", "child_node_data");
    /// let root_group = Group::new(vec![Node::new_group(child_group)], "root", "root_node_data");
    ///
    /// let indexed = root_group.with_idx();
    ///
    /// // Root node has index 0
    /// assert_eq!(indexed.node_data(), utils::with_idx::NodeData { data: &"root_node_data", index: 0 });
    ///
    /// // Child nodes have their respective indices
    /// let children: Vec<_> = indexed.get_children().unwrap().collect();
    /// let lazy_hierarchy::NodeRef::Group(child) = &children[0] else { panic!() };
    ///
    /// assert_eq!(child.node_data(), utils::with_idx::NodeData { data: &"child_node_data", index: 0 });
    ///
    /// let grandchildren: Vec<_> = child.get_children().unwrap().collect();
    /// assert_eq!(grandchildren[0].node_data(), utils::with_idx::NodeData { data: &"node_data1", index: 0 });
    /// assert_eq!(grandchildren[1].node_data(), utils::with_idx::NodeData { data: &"node_data2", index: 1 });
    ///
    /// ```
    fn with_idx(self) -> WithIdxGroupRef<Self> {
        WithIdxGroupRef::new(self)
    }

    /// `format_node` should return just a single line
    ///
    /// Without the feature `colors`, the `colors` argument is ignored
    fn format_as_tree<F: Fn(&NodeRef<Self>) -> String>(
        &self,
        format_node: F,
        colors: bool,
    ) -> impl Display
    where
        Self::StructureErr: Debug,
    {
        struct Displayer<'a, G, F>(&'a G, F, bool);

        impl<'a, G: GroupRef, F: Fn(&NodeRef<G>) -> String> Display for Displayer<'a, G, F>
        where
            G::StructureErr: Debug,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                format_as_tree_rec(
                    &NodeRef::Group(self.0.to_owned()),
                    String::new(),
                    String::new(),
                    &self.1,
                    f,
                    self.2,
                )
            }
        }

        Displayer(self, format_node, colors)
    }
}

/// Without the feature `colors`, the `colors` argument is ignored
fn format_as_tree_rec<G: GroupRef<StructureErr = impl Debug>, F: Fn(&NodeRef<G>) -> String>(
    node: &NodeRef<G>,
    first_line_prefix: String,

    other_lines_prefix: String,
    format_node: &F,
    formatter: &mut std::fmt::Formatter<'_>,
    colors: bool,
) -> Result<(), std::fmt::Error> {
    let node_description = (&format_node)(&node).replace("\n", &format!("\n{other_lines_prefix}"));

    #[cfg(feature = "colors")]
    let node_description = match node {
        NodeRef::Group(_) => Either::Left(node_description.bold()),
        NodeRef::Leaf(_) => Either::Right(node_description.bright_blue()),
    };

    writeln!(formatter, "{first_line_prefix}{node_description}")?;

    if let NodeRef::Group(g) = node {
        match g.get_children() {
            Ok(children) => {
                let mut children = children.collect::<Vec<_>>();
                let last = children.pop();
                for child in children {
                    format_as_tree_rec(
                        &child,
                        format!("{other_lines_prefix} ├─ "),
                        format!("{other_lines_prefix} │  "),
                        format_node,
                        formatter,
                        colors,
                    )?;
                }

                if let Some(last) = last {
                    format_as_tree_rec(
                        &last,
                        format!("{other_lines_prefix} ╰─ "),
                        //format!("{other_lines_prefix} └─ "),
                        format!("{other_lines_prefix}    "),
                        format_node,
                        formatter,
                        colors,
                    )?;
                }
            }
            Err(error) => {
                let msg = format!("╰─ Error obtaining children: {error:?}");
                #[cfg(feature = "colors")]
                let msg = msg.red();
                writeln!(formatter, "{other_lines_prefix} {msg}",)?;
            }
        }
    }

    Ok(())
}

pub trait WithParentUtils<G: GroupRef>: GroupRef<NodeData = WithParentNodeData<G>> {
    /*    /// Reverse the [`with_parent`] operation.
    ///
    /// Common pattern is to call [`with_parent`], then map using the parent and finally get
    /// rid of the parent using [`without_parent`].
    fn without_parent(
        self,
    ) -> impl GroupRef<
        GroupMetadata = Self::GroupMetadata,
        LeafMetadata = Self::LeafMetadata,
        NodeMetadata = G::NodeMetadata,
        StructureErr = Self::StructureErr,
    > {
        self.map_node_data(|node_ref| node_ref.node_data().data)
    }*/
}

pub trait AsGroupRefUtils: AsGroupRef {
    /// Convert to concrete hierarchy by instantiating all the items
    fn collect_to_concrete<'a>(
        &'a self,
    ) -> Result<
        Concrete<
            <Self::GroupRef<'a> as GroupRef>::GroupData,
            <Self::GroupRef<'a> as GroupRef>::LeafData,
            <Self::GroupRef<'a> as GroupRef>::NodeData,
        >,
        <Self::GroupRef<'a> as GroupRef>::StructureErr,
    >
    where
        Self: std::marker::Sized,
    {
        Ok(Concrete::new(self.root().collect_to_concrete()?))
    }
}
