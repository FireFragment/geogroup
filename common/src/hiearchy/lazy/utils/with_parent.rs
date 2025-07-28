//! Creates a new hiearchy where every node holds a [reference](GroupRef) to its parent.

use super::*;

pub fn with_parent<'a, H: hiearchy::lazy::AsGroupRef<'a> + 'a>(hiearchy: &'a H) -> WithParent<'a, H>
where
    <H as hiearchy::lazy::AsGroupRef<'a>>::GroupRef: Clone,
{
    WithParent(hiearchy, PhantomData)
}

#[derive(Clone)]
pub struct WithParent<'a, G: AsGroupRef<'a>>(&'a G, PhantomData<&'a ()>)
where
    <G as hiearchy::lazy::AsGroupRef<'a>>::GroupRef: Clone;

impl<'a, G: AsGroupRef<'a>> AsGroupRef<'a> for WithParent<'a, G>
where
    <G as hiearchy::lazy::AsGroupRef<'a>>::GroupRef: Clone,
{
    type GroupRef = WithParentGroupRef<'a, G::GroupRef>;

    fn root<'s: 'a>(&'s self) -> Self::GroupRef {
        WithParentGroupRef {
            this: self.0.root(),
            parent: None,
            phantom_data: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct WithParentGroupRef<'a, G: GroupRef<'a> + Clone> {
    this: G,
    parent: Option<G>,
    phantom_data: PhantomData<&'a ()>,
}

pub struct WithParentNodeMetadata<'a, G: GroupRef<'a>> {
    pub data: G::NodeMetadata,
    pub parent: Option<G>,
}

pub struct WithParentLeafRef<'a, G: GroupRef<'a> + Clone> {
    this: G::LeafRef,
    parent: Option<G>,
}

impl<'a, G: GroupRef<'a> + Clone> LeafRef for WithParentLeafRef<'a, G> {
    type Metadata = G::LeafMetadata;
    type NodeMetadata = WithParentNodeMetadata<'a, G>;

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

impl<'a, G: GroupRef<'a> + Clone> GroupRef<'a> for WithParentGroupRef<'a, G> {
    type NodeMetadata = WithParentNodeMetadata<'a, G>;
    type LeafMetadata = G::LeafMetadata;
    type GroupMetadata = G::GroupMetadata;
    type StructureErr = G::StructureErr;
    type LeafRef = WithParentLeafRef<'a, G>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<'a, Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        self.this.get_children().map(move |iter| {
            iter.map(|child| match child {
                NodeRef::Group(group) => NodeRef::Group(WithParentGroupRef {
                    this: group,
                    parent: Some(self.this.clone()),
                    phantom_data: PhantomData,
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
