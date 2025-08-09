use std::convert::Infallible;

use derive_more::From;
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
#[derive(Debug)]
pub struct TemplateHiearchy(lazy_hierarchy::Concrete<(), InnerLeafData, NodeData>);

impl From<fs_hierarchy::FolderRef> for TemplateHiearchy {
    fn from(value: fs_hierarchy::FolderRef) -> Self {
        LazyGroup::from(value).into()
    }
}

impl From<LazyGroup> for TemplateHiearchy {
    fn from(value: LazyGroup) -> Self {
        Self(lazy_hierarchy::Concrete::new(
            lazy_hierarchy::concrete::Group::new(
                vec![lazy_hierarchy::concrete::Node::Leaf(
                    lazy_hierarchy::concrete::Leaf::new(
                        InnerLeafData::LazySubgroup(value),
                        NodeData {
                            name: Some(String::from("Root")), // TODO: Translate
                        },
                    ),
                )],
                (),
                NodeData {
                    name: Some(String::from("Root")), // TODO: Translate
                },
            ),
        ))
    }
}

/// May actually represent a [lazy group](LazyGroup)
/// ([subgroup](lazy_hierarchy::fused::MainLeafData::Subgroup) in [`lazy_hierarchy::fused`]'s terms)
#[derive(Debug)]
enum InnerLeafData {
    LazySubgroup(LazyGroup),
    RealLeaf(FileData),
}

#[derive(Debug, From)]
pub enum LazyGroup {
    Fs(fs_hierarchy::FolderRef),
    Sorted(
        algorithm::Sorter<
            ConcreteSortableItem<geo_lib::Point, DateTime<chrono::FixedOffset>, FileData>,
        >,
    ),
}

impl LazyGroup {
    pub fn as_group_ref<'a>(
        &'a self,
    ) -> impl GroupRef<
        GroupData = (),
        LeafData = FileData,
        NodeData = NodeData,
        StructureErr = std::io::Error,
    > + 'a {
        match self {
            LazyGroup::Fs(folder_ref) => Either::Left(
                folder_ref
                    .to_owned()
                    .map_leaf_data(|leaf| FileData {
                        path: leaf.node_data(),
                    })
                    .map_node_data(|node| NodeData {
                        name: node
                            .node_data()
                            .file_name()
                            .map(|name| (*name.to_string_lossy()).to_owned()),
                        /*.unwrap_or_else(|| {
                            log::error!("Path ending in `..`: {:?}", node.node_data());
                            String::new()
                        }),*/
                    }),
            ),
            LazyGroup::Sorted(sorter) => Either::Right(
                sorter
                    .hierarchy()
                    .map_structure_error(|err, _| match err {})
                    .map_node_data(|_| NodeData { name: None })
                    .map_leaf_data(|leaf| leaf.leaf_data().data.to_owned()) // OPT: Possibly needless clone
                    .map_group_data(|_| ()),
            ),
        }
    }
}

impl TemplateHiearchy {
    pub fn root<'a>(
        &'a self,
    ) -> impl lazy_hierarchy::GroupRef<
        GroupData = (),
        LeafData = FileData,
        NodeData = NodeData,
        StructureErr = impl std::error::Error,
    > + 'a {
        use lazy_hierarchy::fused;

        self.0
            .root()
            .map_structure_error(|err, _| match err {})
            .map_group_data(|_| ())
            .map_node_data(|n| n.node_data().to_owned()) // OPT: Possibly needless clone
            .map_leaf_data(|leaf| match leaf.leaf_data() {
                InnerLeafData::LazySubgroup(lazy_group) => {
                    fused::MainLeafData::Subgroup(lazy_group.as_group_ref())
                }
                InnerLeafData::RealLeaf(file_data) => {
                    fused::MainLeafData::RealLeaf(file_data.to_owned()) // OPT: Possibly needless clone
                }
            })
            .fuse()
    }
}

#[derive(Clone, Debug)]
pub struct NodeData {
    pub name: Option<String>,
}
#[derive(Clone, Debug)]
pub struct FileData {
    pub path: PathBuf,
}
