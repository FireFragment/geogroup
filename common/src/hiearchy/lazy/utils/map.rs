use super::*;
pub mod convinience;
pub use convinience::*;

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
pub trait Mapper<'a, Original: AsGroupRef<'a>> {
    type GroupDataNew: 'a;
    type LeafDataNew: 'a;
    type NodeDataNew: 'a;

    /// Transform group metadata into the new group data type.
    ///
    /// This method is called whenever group metadata needs to be accessed
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `group_metadata` - The original group metadata
    /// * `node_metadata` - The associated node metadata for context
    fn map_group_data(
        &self,
        group_metadata: <Original::GroupRef as GroupRef<'a>>::GroupMetadata,
        node_metadata: <Original::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::GroupDataNew;

    /// Transform leaf metadata into the new leaf data type.
    ///
    /// This method is called whenever leaf metadata needs to be accessed
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `leaf_metadata` - The original leaf metadata
    /// * `node_metadata` - The associated node metadata for context
    fn map_leaf_data(
        &self,
        leaf_metadata: <Original::GroupRef as GroupRef<'a>>::LeafMetadata,
        node_metadata: <Original::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::LeafDataNew;

    /// Transform group metadata into node data for group nodes.
    ///
    /// This method is called when accessing node metadata for group nodes
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `group_metadata` - The original group metadata
    /// * `node_metadata` - The associated node metadata for context
    fn map_group_node_data(
        &self,
        group_metadata: <Original::GroupRef as GroupRef<'a>>::GroupMetadata,
        node_metadata: <Original::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::NodeDataNew;

    /// Transform leaf metadata into node data for leaf nodes.
    ///
    /// This method is called when accessing node metadata for leaf nodes
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `leaf_metadata` - The original leaf metadata
    /// * `node_metadata` - The associated node metadata for context
    fn map_leaf_node_data(
        &self,
        leaf_metadata: <Original::GroupRef as GroupRef<'a>>::LeafMetadata,
        node_metadata: <Original::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::NodeDataNew;
}

/// Creates a mapped hierarchy that transforms group data while preserving other metadata types.
///
/// This is a convenience function that creates a `GroupDataMapper` internally, allowing
/// you to transform only group metadata while leaving leaf and node metadata unchanged.
/// This maintains backward compatibility with the original API.
///
/// # Parameters
///
/// * `gr` - The original hierarchy to transform
/// * `fun` - A function that maps group metadata to the new type
///
/// # Returns
///
/// A new hierarchy wrapper that applies the transformation
///
/// # Example
///
/// ```rust
/// let original_hierarchy = /* ... */;
/// let mapped = map_group_data(&original_hierarchy, |group_meta, node_meta| {
///     format!("Transformed: {:?}", group_meta)
/// });
/// ```

pub fn map_group_data<
    'a,
    'orig_gr: 'a,
    OrigGr: AsGroupRef<'a> + 'a,
    GroupDataNew: 'a,
    F: Fn(
            <<OrigGr as AsGroupRef<'a>>::GroupRef as GroupRef<'a>>::GroupMetadata,
            <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
        ) -> GroupDataNew
        + 'a,
>(
    gr: &'orig_gr OrigGr,
    fun: F,
) -> AsMappedGroupRef<'a, OrigGr, GroupDataMapper<F>> {
    AsMappedGroupRef {
        original: gr,
        mapper: GroupDataMapper::new(fun),
    }
}

pub struct AsMappedGroupRef<'a, OrigGr: AsGroupRef<'a>, M: Mapper<'a, OrigGr> + 'a> {
    original: &'a OrigGr,
    mapper: M,
}

impl<'a, OrigGr: AsGroupRef<'a>, M: Mapper<'a, OrigGr> + 'a> AsGroupRef<'a>
    for AsMappedGroupRef<'a, OrigGr, M>
{
    type GroupRef = MappedGroupRef<'a, OrigGr, M>;

    fn root<'s: 'a>(&'s self) -> Self::GroupRef {
        MappedGroupRef {
            this: self.original.root(),
            root: self,
        }
    }
}

pub struct MappedGroupRef<'a, OrigGr: AsGroupRef<'a>, M: Mapper<'a, OrigGr> + 'a> {
    this: OrigGr::GroupRef,
    root: &'a AsMappedGroupRef<'a, OrigGr, M>,
}

pub struct MappedLeafRef<'a, OrigGr: AsGroupRef<'a>, M: Mapper<'a, OrigGr> + 'a> {
    this: <OrigGr::GroupRef as GroupRef<'a>>::LeafRef,
    mapper: &'a M,
}

impl<'a, OrigGr: AsGroupRef<'a>, M: Mapper<'a, OrigGr> + 'a> LeafRef
    for MappedLeafRef<'a, OrigGr, M>
{
    type Metadata = M::LeafDataNew;
    type NodeMetadata = M::NodeDataNew;

    fn leaf_metadata(&self) -> M::LeafDataNew {
        self.mapper
            .map_leaf_data(self.this.leaf_metadata(), self.this.node_metadata())
    }

    fn node_metadata(&self) -> Self::NodeMetadata {
        self.mapper
            .map_leaf_node_data(self.this.leaf_metadata(), self.this.node_metadata())
    }
}

impl<'a, OrigGr: AsGroupRef<'a>, M: Mapper<'a, OrigGr> + 'a> GroupRef<'a>
    for MappedGroupRef<'a, OrigGr, M>
{
    type NodeMetadata = M::NodeDataNew;
    type LeafMetadata = M::LeafDataNew;
    type GroupMetadata = M::GroupDataNew;
    type StructureErr = <OrigGr::GroupRef as GroupRef<'a>>::StructureErr;
    type LeafRef = MappedLeafRef<'a, OrigGr, M>;

    fn get_children(&self) -> Result<impl Iterator<Item = NodeRef<'a, Self>>, Self::StructureErr>
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
        self.root
            .mapper
            .map_group_data(self.this.group_metadata(), self.this.node_metadata())
    }

    fn node_metadata<'b>(&self) -> Self::NodeMetadata {
        self.root
            .mapper
            .map_group_node_data(self.this.group_metadata(), self.this.node_metadata())
    }
}
