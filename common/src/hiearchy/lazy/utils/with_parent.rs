//! Creates a new hiearchy where every node holds a [reference](GroupRef) to its parent.

use super::*;

pub fn with_parent<H: hiearchy::lazy::GroupRef>(hiearchy: H) -> WithParent<H> {
    WithParent(hiearchy)
}

#[derive(Clone)]
pub struct WithParent<G: GroupRef>(G);

impl<G: GroupRef> AsGroupRef for WithParent<G> {
    type GroupRef<'a>
        = WithParentGroupRef<G>
    where
        Self: 'a;

    fn root(&self) -> Self::GroupRef<'_> {
        WithParentGroupRef {
            this: self.0.clone(),
            parent: None,
        }
    }
}

#[derive(Clone)]
pub struct WithParentGroupRef<G: GroupRef> {
    this: G,
    parent: Option<G>,
}

pub struct WithParentNodeMetadata<G: GroupRef> {
    pub data: G::NodeMetadata,
    pub parent: Option<G>,
}

pub struct WithParentLeafRef<G: GroupRef + Clone> {
    this: G::LeafRef,
    parent: Option<G>,
}

impl<G: GroupRef + Clone> LeafRef for WithParentLeafRef<G> {
    type Metadata = G::LeafMetadata;
    type NodeMetadata = WithParentNodeMetadata<G>;

    fn leaf_metadata(&self) -> Self::Metadata {
        self.this.leaf_metadata()
    }

    fn node_metadata(&self) -> Self::NodeMetadata {
        WithParentNodeMetadata {
            data: self.this.node_metadata(),
            parent: self.parent.clone(),
        }
    }
}

impl<G: GroupRef + Clone> GroupRef for WithParentGroupRef<G> {
    type NodeMetadata = WithParentNodeMetadata<G>;
    type LeafMetadata = G::LeafMetadata;
    type GroupMetadata = G::GroupMetadata;
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

    fn group_metadata(&self) -> Self::GroupMetadata {
        self.this.group_metadata()
    }

    fn node_metadata<'b>(&self) -> Self::NodeMetadata {
        WithParentNodeMetadata {
            data: self.this.node_metadata(),
            parent: self.parent.clone(),
        }
    }
}
