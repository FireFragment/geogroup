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
pub trait Mapper<OrigGr: GroupRef> {
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

    /// Transform group metadata into node data for group nodes.
    ///
    /// This method is called when accessing node metadata for group nodes
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `group_ref` - Reference to the original group
    fn map_group_node_data(&self, group_ref: &OrigGr) -> Self::NodeDataNew;

    /// Transform leaf metadata into node data for leaf nodes.
    ///
    /// This method is called when accessing node metadata for leaf nodes
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `leaf_ref` - Reference to the original leaf
    fn map_leaf_node_data(&self, leaf_ref: &<OrigGr as GroupRef>::LeafRef) -> Self::NodeDataNew;
}

/// Creates a mapped hierarchy that transforms group data.
///
/// # Parameters
///
/// * `gr` - The original hierarchy to transform
/// * `fun` - A function that maps group data
pub fn map_group_data<OrigGr: GroupRef, GroupDataNew, F: Fn(&OrigGr) -> GroupDataNew>(
    gr: OrigGr,
    fun: F,
) -> AsMappedGroupRef<OrigGr, GroupDataMapper<F>> {
    AsMappedGroupRef {
        original: gr,
        mapper: GroupDataMapper::new(fun),
    }
}

/// Creates a mapped hierarchy that transforms leaf data.
///
/// # Parameters
///
/// * `gr` - The original hierarchy to transform
/// * `fun` - A function that maps leaf data
pub fn map_leaf_data<
    OrigGr: GroupRef,
    LeafDataNew,
    F: Fn(&<OrigGr as GroupRef>::LeafRef) -> LeafDataNew,
>(
    gr: OrigGr,
    fun: F,
) -> AsMappedGroupRef<OrigGr, LeafDataMapper<F>> {
    AsMappedGroupRef {
        original: gr,
        mapper: LeafDataMapper::new(fun),
    }
}

pub struct AsMappedGroupRef<OrigGr: GroupRef, M: Mapper<OrigGr>> {
    original: OrigGr,
    mapper: M,
}

impl<OrigGr: GroupRef, M: Mapper<OrigGr>> AsGroupRef for AsMappedGroupRef<OrigGr, M> {
    type NodeMetadata<'a>
        = M::NodeDataNew
    where
        Self: 'a;
    type LeafMetadata<'a>
        = M::LeafDataNew
    where
        Self: 'a;
    type GroupMetadata<'a>
        = M::GroupDataNew
    where
        Self: 'a;
    type StructureErr = <OrigGr as GroupRef>::StructureErr;

    type GroupRef<'root>
        = MappedGroupRef<'root, OrigGr, M>
    where
        Self: 'root;

    fn root<'s>(&'s self) -> MappedGroupRef<'s, OrigGr, M> {
        MappedGroupRef {
            this: self.original.clone(),
            root: self,
        }
    }
}

pub struct MappedGroupRef<'root, OrigGr: GroupRef, M: Mapper<OrigGr>> {
    this: OrigGr,
    root: &'root AsMappedGroupRef<OrigGr, M>,
}

impl<'root, OrigGr: GroupRef, M: Mapper<OrigGr>> Clone for MappedGroupRef<'root, OrigGr, M> {
    fn clone(&self) -> Self {
        MappedGroupRef {
            this: self.this.clone(),
            root: self.root,
        }
    }
}

pub struct MappedLeafRef<'root, OrigGr: GroupRef, M: Mapper<OrigGr>> {
    this: OrigGr::LeafRef,
    mapper: &'root M,
}

impl<'root, OrigGr: GroupRef, M: Mapper<OrigGr>> LeafRef for MappedLeafRef<'root, OrigGr, M> {
    type Metadata = M::LeafDataNew;
    type NodeMetadata = M::NodeDataNew;

    fn leaf_metadata(&self) -> M::LeafDataNew {
        self.mapper.map_leaf_data(&self.this)
    }

    fn node_metadata(&self) -> Self::NodeMetadata {
        self.mapper.map_leaf_node_data(&self.this)
    }
}

impl<'a, OrigGr: GroupRef, M: Mapper<OrigGr>> GroupRef for MappedGroupRef<'a, OrigGr, M> {
    type NodeMetadata = M::NodeDataNew;
    type LeafMetadata = M::LeafDataNew;
    type GroupMetadata = M::GroupDataNew;
    type StructureErr = <OrigGr as GroupRef>::StructureErr;
    type LeafRef = MappedLeafRef<'a, OrigGr, M>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        self.this.get_children().map(move |iter| {
            iter.map(|child| match child {
                NodeRef::Group(group) => NodeRef::Group(MappedGroupRef {
                    this: group,
                    root: self.root,
                }),
                NodeRef::Leaf(leaf) => NodeRef::Leaf(MappedLeafRef {
                    this: leaf,
                    mapper: &self.root.mapper,
                }),
            })
        })
    }

    fn group_metadata(&self) -> Self::GroupMetadata {
        self.root.mapper.map_group_data(&self.this)
    }

    fn node_metadata<'b>(&self) -> Self::NodeMetadata {
        self.root.mapper.map_group_node_data(&self.this)
    }
}
