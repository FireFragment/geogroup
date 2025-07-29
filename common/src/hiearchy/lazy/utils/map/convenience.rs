use super::*;

/// A [`Mapper`] implementation that only transforms group data.
pub struct GroupDataMapper<F> {
    map_fn: F,
}

impl<F> GroupDataMapper<F> {
    /// Creates a new `GroupDataMapper` with the given transformation function.
    ///
    /// # Parameters
    ///
    /// * `map_fn` - Function that transforms group metadata and node metadata into new group data
    pub fn new(map_fn: F) -> Self {
        Self { map_fn }
    }
}

impl<OrigGr: GroupRef, GroupDataNew, F: Fn(&OrigGr) -> GroupDataNew> Mapper<OrigGr>
    for GroupDataMapper<F>
{
    type GroupDataNew = GroupDataNew;
    type LeafDataNew = OrigGr::LeafMetadata;
    type NodeDataNew = OrigGr::NodeMetadata;

    fn map_group_data(&self, group_ref: &OrigGr) -> Self::GroupDataNew {
        (self.map_fn)(group_ref)
    }

    fn map_leaf_data(&self, leaf_ref: &OrigGr::LeafRef) -> Self::LeafDataNew {
        leaf_ref.leaf_metadata()
    }

    fn map_group_node_data(&self, group_ref: &OrigGr) -> Self::NodeDataNew {
        group_ref.node_metadata()
    }

    fn map_leaf_node_data(&self, leaf_ref: &OrigGr::LeafRef) -> Self::NodeDataNew {
        leaf_ref.node_metadata()
    }
}

/// A `Mapper` implementation that only transforms leaf data.
pub struct LeafDataMapper<F> {
    map_fn: F,
}

impl<F> LeafDataMapper<F> {
    /// Creates a new `LeafDataMapper` with the given transformation function.
    ///
    /// # Parameters
    ///
    /// * `map_fn` - Function that transforms leaf metadata and node metadata into new leaf data
    pub fn new(map_fn: F) -> Self {
        Self { map_fn }
    }
}

impl<OrigGr: GroupRef, LeafDataNew, F: Fn(&OrigGr::LeafRef) -> LeafDataNew> Mapper<OrigGr>
    for LeafDataMapper<F>
{
    type GroupDataNew = OrigGr::GroupMetadata;
    type LeafDataNew = LeafDataNew;
    type NodeDataNew = OrigGr::NodeMetadata;

    fn map_group_data(&self, group_ref: &OrigGr) -> Self::GroupDataNew {
        group_ref.group_metadata()
    }

    fn map_leaf_data(&self, leaf_ref: &OrigGr::LeafRef) -> Self::LeafDataNew {
        (self.map_fn)(leaf_ref)
    }

    fn map_group_node_data(&self, group_ref: &OrigGr) -> Self::NodeDataNew {
        group_ref.node_metadata()
    }

    fn map_leaf_node_data(&self, leaf_ref: &OrigGr::LeafRef) -> Self::NodeDataNew {
        leaf_ref.node_metadata()
    }
}

pub fn map<OrigGr: GroupRef, M: Mapper<OrigGr>>(
    gr: OrigGr,
    mapper: M,
) -> AsMappedGroupRef<OrigGr, M> {
    AsMappedGroupRef {
        original: gr,
        mapper,
    }
}

pub fn map_as_groupref<'m, OrigGr: GroupRef, M: Mapper<OrigGr>>(
    gr: OrigGr,
    mapper: &'m M,
) -> MappedGroupRef<'m, OrigGr, M> {
    MappedGroupRef { this: gr, mapper }
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
