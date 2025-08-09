//! Utility types for working with hierarchies

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use super::*;
pub mod dissolve;
pub mod map;
pub mod with_parent;
pub use dissolve::Dissolver;
pub use with_parent::{WithParentGroupRef, WithParentLeafRef, WithParentNodeData};

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
        'a,
        LeafDataNew,
        F: Fn(&Self::LeafRef) -> LeafDataNew + 'a + std::clone::Clone,
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
        'a,
        NodeDataNew,
        F: Fn(NodeRef<Self>) -> NodeDataNew + 'a + std::clone::Clone,
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
        'a,
        StructureErrorNew,
        F: Fn(Self::StructureErr, &Self) -> StructureErrorNew + 'a + std::clone::Clone,
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
    fn dissolve_by_key<F: Fn(&Self) -> bool + Clone>(
        self,
        fun_should_dissolve: F,
    ) -> utils::Dissolver<Self, F> {
        utils::Dissolver::new(self, fun_should_dissolve)
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
