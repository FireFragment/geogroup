use super::*;
pub mod convenience;
pub use convenience::*;

/// A trait that defines how to map metadata between different hierarchy representations.
///
/// # Type Parameters
///
/// * `'a` - The lifetime parameter for the hierarchy references
/// * `OrigGr` - The original hierarchy type
///
/// # Associated Types
///
/// * `GroupDataNew` - The new type for group metadata after transformation
/// * `LeafDataNew` - The new type for leaf metadata after transformation
/// * `NodeDataNew` - The new type for node metadata after transformation
pub trait Mapper<OrigGr: GroupRef>: Clone {
    type GroupDataNew;
    type LeafDataNew;
    type NodeDataNew;

    /// Transform group metadata into the new group data type.
    ///
    /// This method is called whenever group metadata needs to be accessed
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `group_ref` - Reference to the original group
    fn map_group_data(&self, group_ref: &OrigGr) -> Self::GroupDataNew;

    /// Transform leaf metadata into the new leaf data type.
    ///
    /// This method is called whenever leaf metadata needs to be accessed
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `leaf_ref` - Reference to the original leaf
    fn map_leaf_data(&self, leaf_ref: &<OrigGr as GroupRef>::LeafRef) -> Self::LeafDataNew;

    /// Transform node data.
    fn map_node_data(&self, node_ref: NodeRef<OrigGr>) -> Self::NodeDataNew;
}

#[derive(Clone, Debug)]
pub struct MappedGroupRef<OrigGr: GroupRef, M: Mapper<OrigGr>> {
    this: OrigGr,
    mapper: M,
}

#[derive(Debug, Clone)]
pub struct MappedLeafRef<OrigGr: GroupRef, M: Mapper<OrigGr>> {
    this: OrigGr::LeafRef,
    mapper: M,
}

impl<'root, OrigGr: GroupRef, M: Mapper<OrigGr>> LeafRef for MappedLeafRef<OrigGr, M> {
    type Metadata = M::LeafDataNew;
    type NodeMetadata = M::NodeDataNew;

    fn leaf_metadata(&self) -> M::LeafDataNew {
        self.mapper.map_leaf_data(&self.this)
    }

    fn node_metadata(&self) -> Self::NodeMetadata {
        self.mapper.map_node_data(NodeRef::Leaf(self.this.clone()))
    }
}

impl<OrigGr: GroupRef, M: Mapper<OrigGr>> GroupRef for MappedGroupRef<OrigGr, M> {
    type NodeMetadata = M::NodeDataNew;
    type LeafMetadata = M::LeafDataNew;
    type GroupMetadata = M::GroupDataNew;
    type StructureErr = <OrigGr as GroupRef>::StructureErr;
    type LeafRef = MappedLeafRef<OrigGr, M>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        self.this.get_children().map(move |iter| {
            iter.map(|child| match child {
                NodeRef::Group(group) => NodeRef::Group(MappedGroupRef {
                    this: group,
                    mapper: self.mapper.clone(),
                }),
                NodeRef::Leaf(leaf) => NodeRef::Leaf(MappedLeafRef {
                    this: leaf,
                    mapper: self.mapper.clone(),
                }),
            })
        })
    }

    fn group_metadata(&self) -> Self::GroupMetadata {
        self.mapper.map_group_data(&self.this)
    }

    fn node_metadata<'b>(&self) -> Self::NodeMetadata {
        self.mapper.map_node_data(NodeRef::Group(self.this.clone()))
    }
}
