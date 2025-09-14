//! See [GroupRefUtils::mark_root]

use super::*;

/// See [GroupRefUtils::mark_root]
#[derive(Debug, Clone)]
pub struct MarkRootGroupRef<G> {
    inner: G,
    is_root: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MarkedRootGroupData<T> {
    pub data: T,
    pub is_root: bool
}

impl<G: GroupRef> MarkRootGroupRef<G> {
    pub fn new(inner: G) -> Self {
        Self {
            inner,
            is_root: true,
        }
    }

    fn child(inner: G) -> Self {
        Self {
            inner,
            is_root: false,
        }
    }
}

impl<G: GroupRef> GroupRef for MarkRootGroupRef<G> {
    type GroupData = MarkedRootGroupData<G::GroupData>;
    type LeafData = G::LeafData;
    type NodeData = G::NodeData;
    type StructureErr = G::StructureErr;
    type LeafRef = G::LeafRef;

    fn group_data(&self) -> Self::GroupData {
        MarkedRootGroupData {
            is_root: self.is_root,
            data: self.inner.group_data()
        }
    }

    fn node_data(&self) -> Self::NodeData {
        self.inner.node_data()
    }

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr> {
        let children = self.inner.get_children()?;
        Ok(children.map(|child| match child {
            NodeRef::Group(g) => NodeRef::Group(Self::child(g)),
            NodeRef::Leaf(l) => NodeRef::Leaf(l),
        }))
    }
}

/// Create a hierarchy adapter that marks the root group with a boolean flag
pub fn mark_root<G: GroupRef>(group: G) -> MarkRootGroupRef<G> {
    MarkRootGroupRef::new(group)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_root_functionality() {
        use crate::hiearchy::concrete::{Group, Leaf, Node};

        // Create a simple test hierarchy
        let leaf1 = Leaf::new("leaf1_data", "leaf1_node");
        let leaf2 = Leaf::new("leaf2_data", "leaf2_node");

        let child_group = Group::new(
            vec![Node::new_leaf(leaf2)],
            "child_group_data",
            "child_node_data",
        );

        let root_group = Group::new(
            vec![
                Node::new_leaf(leaf1),
                Node::new_group(child_group),
            ],
            "root_group_data",
            "root_node_data",
        );

        // Apply mark_root transformation
        let marked = mark_root(&root_group);

        assert_eq!(marked.group_data(), MarkedRootGroupData { is_root: true, data: &"root_group_data" });

        let children: Vec<_> = marked.get_children().unwrap().collect();
        assert_eq!(children.len(), 2);

        if let NodeRef::Group(child) = &children[1] {
            assert_eq!(child.group_data(), MarkedRootGroupData { is_root: false, data: &"child_group_data" });
        } else {
            panic!("Expected second child to be a group");
        }
    }
}
