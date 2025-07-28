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
    /// * `group_ref` - Reference to the original group
    fn map_group_data(&self, group_ref: &Original::GroupRef) -> Self::GroupDataNew;

    /// Transform leaf metadata into the new leaf data type.
    ///
    /// This method is called whenever leaf metadata needs to be accessed
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `leaf_ref` - Reference to the original leaf
    fn map_leaf_data(
        &self,
        leaf_ref: &<Original::GroupRef as GroupRef>::LeafRef,
    ) -> Self::LeafDataNew;

    /// Transform group metadata into node data for group nodes.
    ///
    /// This method is called when accessing node metadata for group nodes
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `group_ref` - Reference to the original group
    fn map_group_node_data(&self, group_ref: &Original::GroupRef) -> Self::NodeDataNew;

    /// Transform leaf metadata into node data for leaf nodes.
    ///
    /// This method is called when accessing node metadata for leaf nodes
    /// in the transformed hierarchy.
    ///
    /// # Parameters
    ///
    /// * `leaf_ref` - Reference to the original leaf
    fn map_leaf_node_data(
        &self,
        leaf_ref: &<Original::GroupRef as GroupRef>::LeafRef,
    ) -> Self::NodeDataNew;
}

/// Creates a mapped hierarchy that transforms group data.
///
/// # Parameters
///
/// * `gr` - The original hierarchy to transform
/// * `fun` - A function that maps group data
pub fn map_group_data<
    'a,
    'orig_gr: 'a,
    OrigGr: AsGroupRef<'a> + 'a,
    GroupDataNew: 'a,
    F: Fn(&<OrigGr as AsGroupRef<'a>>::GroupRef) -> GroupDataNew + 'a,
>(
    gr: &'orig_gr OrigGr,
    fun: F,
) -> AsMappedGroupRef<'a, OrigGr, GroupDataMapper<F>> {
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
    'a,
    'orig_gr: 'a,
    OrigGr: AsGroupRef<'a> + 'a,
    LeafDataNew: 'a,
    F: Fn(&<<OrigGr as AsGroupRef<'a>>::GroupRef as GroupRef>::LeafRef) -> LeafDataNew + 'a,
>(
    gr: &'orig_gr OrigGr,
    fun: F,
) -> AsMappedGroupRef<'a, OrigGr, LeafDataMapper<F>> {
    AsMappedGroupRef {
        original: gr,
        mapper: LeafDataMapper::new(fun),
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
    this: <OrigGr::GroupRef as GroupRef>::LeafRef,
    mapper: &'a M,
}

impl<'a, OrigGr: AsGroupRef<'a>, M: Mapper<'a, OrigGr> + 'a> LeafRef
    for MappedLeafRef<'a, OrigGr, M>
{
    type Metadata = M::LeafDataNew;
    type NodeMetadata = M::NodeDataNew;

    fn leaf_metadata(&self) -> M::LeafDataNew {
        self.mapper.map_leaf_data(&self.this)
    }

    fn node_metadata(&self) -> Self::NodeMetadata {
        self.mapper.map_leaf_node_data(&self.this)
    }
}

impl<'a, OrigGr: AsGroupRef<'a>, M: Mapper<'a, OrigGr> + 'a> GroupRef
    for MappedGroupRef<'a, OrigGr, M>
{
    type NodeMetadata = M::NodeDataNew;
    type LeafMetadata = M::LeafDataNew;
    type GroupMetadata = M::GroupDataNew;
    type StructureErr = <OrigGr::GroupRef as GroupRef>::StructureErr;
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
