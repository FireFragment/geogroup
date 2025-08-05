use std::convert::Infallible;

use lazy_hierarchy::GroupRef;

use super::*;

/// A state of the hiearchy.
/// It doesn't represent a concrete hiearchy, but rather abstract template for constructing such a hiearchy.
/// This is how hiearchy is represented in RAM and saved in projects.
///
/// For example, it contains all the "locked" groups' contents, but the automatically sorted ones are
/// represented just by configuration of the sorting algorithm.
///
/// Storing it this way has numerous advantages over storing the entire hiearchy:
///     - More closely represents the intended mental model of using the program, that is, if
///       a group is said to be sorted automatically, its contents are not a fixed hiearchy,
///       but a dynamic one, automatically updated by an algorithm.
///     - Allows for changing the list of files to automatically include the in the hiearchy.
///     - Reading specific parts of the hiearchy without processing its entire contents.
///     - Lower RAM usage.
///
/// If you are interested in examining the actual structure of hiearchy, see [`hiearchy::lazy`]
pub struct TemplateHiearchy<'a>(
    lazy_hierarchy::concrete::ConcreteHiearchy<(), InnerLeafData<'a>, &'a NodeData>,
);

/// May actually represent a [lazy group](LazyGroup)
/// ([subgroup](lazy_hierarchy::fused::MainLeafData::Subgroup) in [`lazy_hierarchy::fused`]'s terms)
enum InnerLeafData<'a> {
    LazySubgroup(LazyGroup<'a>),
    RealLeaf(&'a FileData),
}

pub enum LazyGroup<'a> {
    Fs(fs_hierarchy::FolderRef),
    Sorted(
        algorithm::Sorter<
            ConcreteSortableItem<geo_lib::Point, DateTime<chrono::FixedOffset>, &'a FileData>,
        >,
    ),
}

impl<'a> LazyGroup<'a> {
    pub fn as_group_ref(
        &'a self,
    ) -> impl GroupRef<
        GroupData = (),
        LeafData = &'a FileData,
        NodeData = &'a NodeData,
        StructureErr = Infallible,
    > + 'a {
        match self {
            LazyGroup::Fs(folder_ref) => Either::Left(folder_ref.to_owned()),
            LazyGroup::Sorted(sorter) => Either::Right(sorter.hierarchy()),
        }
    }
}

impl<'a> TemplateHiearchy<'a> {
    pub fn root(
        &'a self,
    ) -> impl lazy_hierarchy::GroupRef<GroupData = (), LeafData = &'a FileData, NodeData = &'a NodeData>
    {
        use lazy_hierarchy::fused;

        self.0
            .root()
            .map_group_data(|_| ())
            .map_node_data(|n| *n.node_data())
            .map_leaf_data(|leaf| match leaf.leaf_data() {
                InnerLeafData::LazySubgroup(lazy_group) => {
                    fused::MainLeafData::Subgroup(lazy_group.as_group_ref())
                }
                InnerLeafData::RealLeaf(file_data) => fused::MainLeafData::RealLeaf(*file_data),
            })
            .fuse()
    }
}

pub struct NodeData {
    pub name: String,
}

pub struct FileData {
    pub path: PathBuf,
    pub name: String,
}
