//! Creates a new hiearchy where every node holds a [reference](GroupRef) to its parent.

use super::*;

pub fn with_parent<H: GroupRef>(hiearchy: H) -> WithParentGroupRef<H> {
    WithParentGroupRef {
        this: hiearchy,
        parent: None,
    }
}

#[derive(Clone, Debug)]
pub struct WithParentGroupRef<G: GroupRef> {
    this: G,
    parent: Option<G>,
}

#[derive(Clone, Debug)]
pub struct WithParentNodeData<G: GroupRef> {
    pub data: G::NodeData,
    pub parent: Option<G>,
}

#[derive(Clone, Debug)]
pub struct WithParentLeafRef<G: GroupRef + Clone> {
    this: G::LeafRef,
    parent: Option<G>,
}

impl<G: GroupRef + Clone> LeafRef for WithParentLeafRef<G> {
    type LeafData = G::LeafData;
    type NodeData = WithParentNodeData<G>;

    fn leaf_data(&self) -> Self::LeafData {
        self.this.leaf_data()
    }

    fn node_data(&self) -> Self::NodeData {
        WithParentNodeData {
            data: self.this.node_data(),
            parent: self.parent.clone(),
        }
    }
}

impl<G: GroupRef + Clone> GroupRef for WithParentGroupRef<G> {
    type NodeData = WithParentNodeData<G>;
    type LeafData = G::LeafData;
    type GroupData = G::GroupData;
    type StructureErr = G::StructureErr;
    type LeafRef = WithParentLeafRef<G>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        self.this.get_children().map(move |iter| {
            iter.map(|child| match child {
                NodeRef::Group(group) => NodeRef::Group(WithParentGroupRef {
                    this: group,
                    parent: Some(self.this.clone()),
                }),
                NodeRef::Leaf(leaf) => NodeRef::Leaf(WithParentLeafRef {
                    this: leaf,
                    parent: Some(self.this.clone()),
                }),
            })
        })
    }

    fn group_data(&self) -> Self::GroupData {
        self.this.group_data()
    }

    fn node_data<'b>(&self) -> Self::NodeData {
        WithParentNodeData {
            data: self.this.node_data(),
            parent: self.parent.clone(),
        }
    }
}
