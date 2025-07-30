use super::*;

/// A [`Mapper`] implementation that only transforms group data.
///
/// Shouldn't be super expensive to [clone](Clone::clone), it's cloned whenever a new
/// [MappedGroupRef] is created.
#[derive(Clone)]
pub struct GroupDataMapper<F: Clone> {
    map_fn: F,
}

impl<F: Clone> GroupDataMapper<F> {
    /// Creates a new `GroupDataMapper` with the given transformation function.
    ///
    /// # Parameters
    ///
    /// * `map_fn` - Function that transforms group data and node data into new group data
    pub fn new(map_fn: F) -> Self {
        Self { map_fn }
    }
}

impl<OrigGr: GroupRef, GroupDataNew, F: Fn(&OrigGr) -> GroupDataNew + Clone> Mapper<OrigGr>
    for GroupDataMapper<F>
{
    type GroupDataNew = GroupDataNew;
    type LeafDataNew = OrigGr::LeafData;
    type NodeDataNew = OrigGr::NodeData;

    fn map_group_data(&self, group_ref: &OrigGr) -> Self::GroupDataNew {
        (self.map_fn)(group_ref)
    }

    fn map_leaf_data(&self, leaf_ref: &OrigGr::LeafRef) -> Self::LeafDataNew {
        leaf_ref.leaf_data()
    }

    fn map_node_data(&self, node_ref: NodeRef<OrigGr>) -> Self::NodeDataNew {
        node_ref.node_data()
    }
}

/// A `Mapper` implementation that only transforms leaf data.
#[derive(Clone)]
pub struct LeafDataMapper<F> {
    map_fn: F,
}

impl<F> LeafDataMapper<F> {
    /// Creates a new `LeafDataMapper` with the given transformation function.
    ///
    /// # Parameters
    ///
    /// * `map_fn` - Function that transforms leaf data and node data into new leaf data
    pub fn new(map_fn: F) -> Self {
        Self { map_fn }
    }
}

impl<OrigGr: GroupRef, LeafDataNew, F: Fn(&OrigGr::LeafRef) -> LeafDataNew + Clone> Mapper<OrigGr>
    for LeafDataMapper<F>
{
    type GroupDataNew = OrigGr::GroupData;
    type LeafDataNew = LeafDataNew;
    type NodeDataNew = OrigGr::NodeData;

    fn map_group_data(&self, group_ref: &OrigGr) -> Self::GroupDataNew {
        group_ref.group_data()
    }

    fn map_leaf_data(&self, leaf_ref: &OrigGr::LeafRef) -> Self::LeafDataNew {
        (self.map_fn)(leaf_ref)
    }

    fn map_node_data(&self, node_ref: NodeRef<OrigGr>) -> Self::NodeDataNew {
        node_ref.node_data()
    }
}

#[derive(Clone)]
pub struct NodeDataMapper<F> {
    map_fn: F,
}

impl<F> NodeDataMapper<F> {
    /// Creates a new `LeafDataMapper` with the given transformation function.
    ///
    /// # Parameters
    ///
    /// * `map_fn` - Function that transforms leaf data and node data into new leaf data
    pub fn new(map_fn: F) -> Self {
        Self { map_fn }
    }
}

impl<OrigGr: GroupRef, NodeDataNew, F: Fn(NodeRef<OrigGr>) -> NodeDataNew + Clone> Mapper<OrigGr>
    for NodeDataMapper<F>
{
    type GroupDataNew = OrigGr::GroupData;
    type LeafDataNew = OrigGr::LeafData;
    type NodeDataNew = NodeDataNew;

    fn map_group_data(&self, group_ref: &OrigGr) -> Self::GroupDataNew {
        group_ref.group_data()
    }

    fn map_leaf_data(&self, leaf_ref: &OrigGr::LeafRef) -> Self::LeafDataNew {
        leaf_ref.leaf_data()
    }

    fn map_node_data(&self, node_ref: NodeRef<OrigGr>) -> Self::NodeDataNew {
        (self.map_fn)(node_ref)
    }
}

pub fn map<OrigGr: GroupRef, M: Mapper<OrigGr>>(
    gr: OrigGr,
    mapper: M,
) -> MappedGroupRef<OrigGr, M> {
    MappedGroupRef { this: gr, mapper }
}

pub fn map_as_groupref<'m, OrigGr: GroupRef, M: Mapper<OrigGr>>(
    gr: OrigGr,
    mapper: M,
) -> MappedGroupRef<OrigGr, M> {
    MappedGroupRef { this: gr, mapper }
}

/// Creates a mapped hierarchy that transforms group data.
///
/// # Parameters
///
/// * `gr` - The original hierarchy to transform
/// * `fun` - A function that maps group data
pub fn map_group_data<OrigGr: GroupRef, GroupDataNew, F: Fn(&OrigGr) -> GroupDataNew + Clone>(
    gr: OrigGr,
    fun: F,
) -> MappedGroupRef<OrigGr, GroupDataMapper<F>> {
    MappedGroupRef {
        this: gr,
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
    F: Fn(&<OrigGr as GroupRef>::LeafRef) -> LeafDataNew + Clone,
>(
    gr: OrigGr,
    fun: F,
) -> MappedGroupRef<OrigGr, LeafDataMapper<F>> {
    MappedGroupRef {
        this: gr,
        mapper: LeafDataMapper::new(fun),
    }
}

/// Creates a mapped hierarchy that transforms node data.
///
/// # Parameters
///
/// * `gr` - The original hierarchy to transform
/// * `fun` - A function that maps node data
pub fn map_node_data<
    OrigGr: GroupRef,
    NodeDataNew,
    F: Fn(NodeRef<OrigGr>) -> NodeDataNew + Clone,
>(
    gr: OrigGr,
    fun: F,
) -> MappedGroupRef<OrigGr, NodeDataMapper<F>> {
    MappedGroupRef {
        this: gr,
        mapper: NodeDataMapper::new(fun),
    }
}
