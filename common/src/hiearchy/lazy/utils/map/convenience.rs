use super::*;

/// A `Mapper` implementation that only transforms group data.
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
    F: Fn(
            <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
            <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
        ) -> GroupDataNew
        + 'a,
    <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata: 'a,
    <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata: 'a,
{
    type GroupDataNew = GroupDataNew;
    type LeafDataNew = <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata;
    type NodeDataNew = <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata;

    fn map_group_data(
        &self,
        group_metadata: <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
        node_metadata: <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::GroupDataNew {
        (self.map_fn)(group_metadata, node_metadata)
    }

    fn map_leaf_data(
        &self,
        leaf_metadata: <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        _node_metadata: <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::LeafDataNew {
        leaf_metadata
    }

    fn map_group_node_data(
        &self,
        _group_metadata: <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
        node_metadata: <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::NodeDataNew {
        node_metadata
    }

    fn map_leaf_node_data(
        &self,
        _leaf_metadata: <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        node_metadata: <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::NodeDataNew {
        node_metadata
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
    F: Fn(
            <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
            <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
        ) -> LeafDataNew
        + 'a,
    <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata: 'a,
    <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata: 'a,
{
    type GroupDataNew = <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata;
    type LeafDataNew = LeafDataNew;
    type NodeDataNew = <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata;

    fn map_group_data(
        &self,
        group_metadata: <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
        _node_metadata: <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::GroupDataNew {
        group_metadata
    }

    fn map_leaf_data(
        &self,
        leaf_metadata: <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        node_metadata: <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::LeafDataNew {
        (self.map_fn)(leaf_metadata, node_metadata)
    }

    fn map_group_node_data(
        &self,
        _group_metadata: <OrigGr::GroupRef as GroupRef<'a>>::GroupMetadata,
        node_metadata: <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::NodeDataNew {
        node_metadata
    }

    fn map_leaf_node_data(
        &self,
        _leaf_metadata: <OrigGr::GroupRef as GroupRef<'a>>::LeafMetadata,
        node_metadata: <OrigGr::GroupRef as GroupRef<'a>>::NodeMetadata,
    ) -> Self::NodeDataNew {
        node_metadata
    }
}
