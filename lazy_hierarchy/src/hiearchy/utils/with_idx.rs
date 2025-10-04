use super::*;

/// Helper struct that implements the index tracking functionality
#[derive(Clone, Debug)]
pub struct WithIdxGroupRef<G: GroupRef> {
    inner: G,
    index: usize,
}

impl<G: GroupRef> WithIdxGroupRef<G> {
    pub fn new(inner: G) -> Self {
        Self { inner, index: 0 }
    }
}

#[derive(Clone, Debug)]
pub struct WithIdxLeafRef<G: GroupRef> {
    inner: G::LeafRef,
    index: usize,
}

impl<G: GroupRef> LeafRef for WithIdxLeafRef<G> {
    type LeafData = G::LeafData;
    type NodeData = NodeData<G::NodeData>;

    fn leaf_data(&self) -> Self::LeafData {
        self.inner.leaf_data()
    }

    fn node_data(&self) -> Self::NodeData {
        NodeData::new(self.inner.node_data(), self.index)
    }
}

impl<G: GroupRef> GroupRef for WithIdxGroupRef<G> {
    type NodeData = NodeData<G::NodeData>;
    type LeafData = G::LeafData;
    type GroupData = G::GroupData;
    type StructureErr = G::StructureErr;
    type LeafRef = WithIdxLeafRef<G>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        self.inner.get_children().map(|iter| {
            iter.enumerate().map(|(index, child)| match child {
                NodeRef::Group(group) => NodeRef::Group(WithIdxGroupRef { inner: group, index }),
                NodeRef::Leaf(leaf) => NodeRef::Leaf(WithIdxLeafRef {
                    inner: leaf,
                    index,
                }),
            })
        })
    }

    fn group_data(&self) -> Self::GroupData {
        self.inner.group_data()
    }

    fn node_data(&self) -> Self::NodeData {
        NodeData::new(self.inner.node_data(), self.index)
    }
}


/// Node data wrapper that includes the index of the node within its parent group
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeData<T> {
    /// The original node data
    pub data: T,
    /// The index of this node within its parent group
    pub index: usize,
}

impl<T> NodeData<T> {
    pub fn new(data: T, index: usize) -> Self {
        Self { data, index }
    }
}
