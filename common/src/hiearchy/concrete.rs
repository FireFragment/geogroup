use crate::*;
use core::fmt;
use std::{collections::HashMap, convert::Infallible};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GroupRef {
    Root,
    Id(u64),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LeafRef(u64);
use hiearchy::lazy::LoadingResult;
pub use hiearchy::lazy::NodeRef;

#[derive(Debug, Clone)]
pub struct ConcreteHiearchy<LeafData: Clone, GroupData: Clone = ()> {
    leaves: HashMap<LeafRef, LeafData>,
    groups: HashMap<u64, (GroupData, Vec<NodeRef<Self>>)>,
    /// To safely ensure there's always some root group
    root_group: (GroupData, Vec<NodeRef<Self>>),

    /// Key for which it's garantueed, that any higher key is not used for [`Self::leaves`] nor [`Self::groups`]
    highest_used: u64,
}

impl<LeafData: Clone, GroupData: Clone> ConcreteHiearchy<LeafData, GroupData> {
    pub fn new(root_group_data: GroupData) -> Self {
        Self {
            leaves: HashMap::new(),
            groups: HashMap::new(),
            root_group: (root_group_data, Vec::new()),
            highest_used: 0,
        }
    }

    fn get_group_by_id_mut(
        &mut self,
        parent_group: GroupRef,
    ) -> &mut (GroupData, Vec<NodeRef<Self>>) {
        match parent_group {
            GroupRef::Root => &mut self.root_group,
            GroupRef::Id(id) => self.groups.get_mut(&id).expect("Group ID not found"),
        }
    }

    /// Returns id of the new group
    pub fn add_subgroup(&mut self, parent_group: GroupRef, subgroup: GroupData) -> GroupRef {
        self.highest_used += 1;
        let new_id = self.highest_used;
        self.groups
            .insert(new_id, (subgroup, Vec::new()))
            .expect("There was a group with key highest_unused");
        let new_group_ref = GroupRef::Id(new_id);
        self.get_group_by_id_mut(parent_group)
            .1
            .push(NodeRef::Group(new_group_ref.clone()));
        new_group_ref
    }

    pub fn add_leaf(&mut self, parent_group: GroupRef, leaf: LeafData) -> LeafRef {
        self.highest_used += 1;
        let new_id = self.highest_used;
        let new_leaf_ref = LeafRef(new_id);
        self.leaves.insert(new_leaf_ref.clone(), leaf);
        self.get_group_by_id_mut(parent_group)
            .1
            .push(NodeRef::Leaf(new_leaf_ref.clone()));
        new_leaf_ref
    }
}

impl<LeafData: Clone, GroupData: Clone> hiearchy::Lazy for ConcreteHiearchy<LeafData, GroupData> {
    type GroupRef = GroupRef;
    type LeafRef = LeafRef;
    type GroupMetadata = GroupData;
    type LeafMetadata = LeafData;
    type NodeMetadata = ();
    type StructureErr = Infallible;
    type DoesLoading = Infallible;

    fn root(&self) -> Self::GroupRef {
        GroupRef::Root
    }

    fn get_children(
        &self,
        group: Self::GroupRef,
    ) -> hiearchy::lazy::LoadingResult<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    {
        LoadingResult::new_ok(match group {
            GroupRef::Root => self.root_group.1.iter().cloned(),
            GroupRef::Id(id) => self.groups[&id].1.iter().cloned(),
        })
    }

    fn node_metadata(&self, _node: hiearchy::lazy::NodeRef<Self>) -> Self::NodeMetadata {}

    fn group_metadata(&self, group: Self::GroupRef) -> Self::GroupMetadata {
        match group {
            GroupRef::Root => self.root_group.0.clone(),
            GroupRef::Id(id) => self.groups[&id].0.clone(),
        }
    }

    fn leaf_metadata(&self, leaf: Self::LeafRef) -> Self::LeafMetadata {
        self.leaves[&leaf].clone()
    }
}

#[derive(Debug, Clone)]
pub enum Node<LeafData, GroupData = ()> {
    Group(Vec<Node<LeafData, GroupData>>, GroupData),
    Item(LeafData),
}

// TODO: Remove theese
impl<L, G> Node<L, G> {
    #[deprecated]
    pub fn map_leafs<Out, Fun: Fn(L) -> Out>(self, fun: &Fun) -> Node<Out, G> {
        match self {
            Node::Group(group, data) => Node::Group(
                group.into_iter().map(|item| item.map_leafs(fun)).collect(),
                data,
            ),
            Node::Item(it) => Node::Item(fun(it)),
        }
    }
    #[deprecated]
    pub fn map_group_data<Out, Fun: Fn(G) -> Out>(self, fun: &Fun) -> Node<L, Out> {
        match self {
            Node::Group(group, data) => Node::Group(
                group
                    .into_iter()
                    .map(|item| item.map_group_data(fun))
                    .collect(),
                fun(data),
            ),
            Node::Item(it) => Node::Item(it),
        }
    }
    pub fn leaves_mut<'s>(&'s mut self) -> Box<dyn Iterator<Item = &mut L> + 's> {
        match self {
            Node::Group(g, _) => Box::new(g.iter_mut().flat_map(|item| item.leaves_mut())),
            Node::Item(it) => Box::new(std::iter::once(it)),
        }
    }
    pub fn leaves<'s>(&'s self) -> Box<dyn Iterator<Item = &L> + 's> {
        match self {
            Node::Group(g, _) => Box::new(g.iter().flat_map(|item| item.leaves())),
            Node::Item(it) => Box::new(std::iter::once(it)),
        }
    }
    pub fn leaves_cloned<'s>(&'s self) -> impl Iterator<Item = L>
    where
        L: Clone,
    {
        match self {
            Node::Group(g, _) => g
                .iter()
                .map(|item| item.leaves_cloned())
                .flatten()
                .collect::<Vec<_>>()
                .into_iter(),
            Node::Item(it) => vec![it.to_owned()].into_iter(),
        }
    }
}

impl<L: fmt::Display, G: fmt::Debug> Node<L, G> {
    /// Improvised and doesn't look good yet. Should be only used for debugging
    pub fn print_tree(&self) -> String {
        match self {
            Self::Group(group, data) => format!(
                "--* {data:?}\n{}",
                group
                    .iter()
                    .map(|subitem| subitem.print_tree().replace("\n", "\n  |"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            Self::Item(item) => format!("--{}", item),
        }
    }
}

impl<F: fmt::Display, D, G: fmt::Debug> Node<(F, D), G> {
    /// Improvised and doesn't look good yet. Should be only used for debugging
    pub fn print_tree_points_only(&self) -> String {
        match self {
            Self::Group(group, data) => format!(
                "--* {data:?}\n{}",
                group
                    .iter()
                    .map(|subitem| subitem.print_tree_points_only().replace("\n", "\n  |"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            Self::Item(item) => format!("--{}", item.0),
        }
    }
}
