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

impl<'a, OrigGr: AsGroupRef<'a>, GroupDataNew: 'a, F> Mapper<'a, OrigGr> for GroupDataMapper<F>
where
    F: Fn(&<OrigGr as AsGroupRef<'a>>::GroupRef) -> GroupDataNew + 'a,
    <OrigGr::GroupRef as GroupRef>::LeafMetadata: 'a,
    <OrigGr::GroupRef as GroupRef>::NodeMetadata: 'a,
{
    type GroupDataNew = GroupDataNew;
    type LeafDataNew = <OrigGr::GroupRef as GroupRef>::LeafMetadata;
    type NodeDataNew = <OrigGr::GroupRef as GroupRef>::NodeMetadata;

    fn map_group_data(&self, group_ref: &OrigGr::GroupRef) -> Self::GroupDataNew {
        (self.map_fn)(group_ref)
    }

    fn map_leaf_data(
        &self,
        leaf_ref: &<OrigGr::GroupRef as GroupRef>::LeafRef,
    ) -> Self::LeafDataNew {
        leaf_ref.leaf_metadata()
    }

    fn map_group_node_data(&self, group_ref: &OrigGr::GroupRef) -> Self::NodeDataNew {
        group_ref.node_metadata()
    }

    fn map_leaf_node_data(
        &self,
        leaf_ref: &<OrigGr::GroupRef as GroupRef>::LeafRef,
    ) -> Self::NodeDataNew {
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

impl<'a, OrigGr: AsGroupRef<'a>, LeafDataNew: 'a, F> Mapper<'a, OrigGr> for LeafDataMapper<F>
where
    F: Fn(&<OrigGr::GroupRef as GroupRef>::LeafRef) -> LeafDataNew + 'a,
    <OrigGr::GroupRef as GroupRef>::GroupMetadata: 'a,
    <OrigGr::GroupRef as GroupRef>::NodeMetadata: 'a,
{
    type GroupDataNew = <OrigGr::GroupRef as GroupRef>::GroupMetadata;
    type LeafDataNew = LeafDataNew;
    type NodeDataNew = <OrigGr::GroupRef as GroupRef>::NodeMetadata;

    fn map_group_data(&self, group_ref: &OrigGr::GroupRef) -> Self::GroupDataNew {
        group_ref.group_metadata()
    }

    fn map_leaf_data(
        &self,
        leaf_ref: &<OrigGr::GroupRef as GroupRef>::LeafRef,
    ) -> Self::LeafDataNew {
        (self.map_fn)(leaf_ref)
    }

    fn map_group_node_data(&self, group_ref: &OrigGr::GroupRef) -> Self::NodeDataNew {
        group_ref.node_metadata()
    }

    fn map_leaf_node_data(
        &self,
        leaf_ref: &<OrigGr::GroupRef as GroupRef>::LeafRef,
    ) -> Self::NodeDataNew {
        leaf_ref.node_metadata()
    }
}
