use std::convert::Infallible;
use derive_more::From;
use lazy_hierarchy::{concrete::Leaf, GroupRef};

use super::*;


pub mod lazy_group;

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
pub struct TemplateHiearchy(pub lazy_hierarchy::Concrete<(), InnerLeafData, NodeData>);

impl From<fs_hierarchy::FolderRef> for TemplateHiearchy {
    fn from(value: fs_hierarchy::FolderRef) -> Self {
        lazy_group::Dynamic::from(lazy_group::Final::from(value)).into()
    }
}

impl From<lazy_group::Dynamic> for TemplateHiearchy {
    fn from(value: lazy_group::Dynamic) -> Self {
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

/// May actually represent a [lazy group](lazy_group::Dynamic)
/// ([subgroup](lazy_hierarchy::fused::MainLeafData::Subgroup) in [`lazy_hierarchy::fused`]'s terms)
#[derive(Debug)]
pub enum InnerLeafData {
    LazySubgroup(lazy_group::Dynamic),
    RealLeaf(FileData),
}


#[derive(Debug)]
pub struct LGSorted {
    item_source: ItemSource,
    sorter: algorithm::Sorter<
        ConcreteSortableItem<geo_lib::Point, DateTime<chrono::FixedOffset>, FileData>,
    >,
}

impl LGSorted {
    /// Potentially long-running. Returns [None] if terminated by progress_callback
    pub fn new(item_source: ItemSource, params: algorithm::Params, progress_callback: &mut ProgressCallback) -> Option<Self> {
        let items = item_source.clone()
            .into_concrete()
            .with_progress_callback(
                progress_callback,
                |entry| {
                    entry
                        .as_ref()
                        .map_or_else(|err| err.path(), |entry| Some(entry))
                        .map(|path| format!("Listing files: {}", path.to_string_lossy().to_string()))
                },
                100,
            )
            .collect::<Vec<_>>().into_iter() // Used to make progress reporting more accurate
            .with_progress_callback(
                progress_callback,
                |entry| {
                    entry
                        .as_ref()
                        .map_or_else(|err| err.path(), |entry| Some(entry))
                        .map(|path| format!("Reading metadata: {}", path.to_string_lossy().to_string()))
                },
                10,
            ).filter_map(|entry| entry.ok()) // TODO: Don't ignore errors
            .filter_map(|path| {
                loaders::GeneralLoader.get_data(path.as_ref()).ok()?.as_sortable_item(FileData { path }).ok() // TODO: Don't ignore errors
            })
            .collect();

        log::debug!("LGSorted: Loaded all files");
        let should_terminate = progress_callback.call(None, Some("Sorting"));
        if should_terminate {
            return None;
        }
        Some(Self {
            // TODO: Add progress callback
            sorter: algorithm::Sorter::new(
                items,
                params
            ),  // TODO: Don't ignore errors
            item_source,
        })
    }

    pub fn params_mut(&mut self) -> &mut algorithm::Params {
        self.sorter.params_mut()
    }
}

#[derive(Debug, Clone)]
pub enum StructureErr<E: std::error::Error> {
    Pending,
    Error(E),
}

impl TemplateHiearchy {
    /// Converts to [lazy_hierarchy::GroupRef] representing the "final" hierarchy,
    /// ie. the form of the hierarchy to be displayed to the user. This is in contrast with [`self.0`](TemplateHiearchy::0)
    /// which is the "template" for creating the "final" hierarchy.
    ///
    /// Note that groups returned by this function do NOT have 1:1 correspondence with folders
    /// to be applied, see [GroupData::LazySubgroupRoot]
    pub fn root_final<'a>(
        &'a self,
    ) -> impl lazy_hierarchy::GroupRef<
        GroupData = GroupData,
        LeafData = LeafData,
        NodeData = NodeData,
        StructureErr = StructureErr<impl std::error::Error>,
    > + 'a {
        use lazy_hierarchy::fused;

        self.0
            .root()
            .map_structure_error(|err, _| match err {})
            .map_group_data(|_| GroupData::Static)
            .map_node_data(|n| n.node_data().to_owned()) // OPT: Possibly needless clone
            .map_leaf_data(|leaf| match leaf.leaf_data() {
                InnerLeafData::LazySubgroup(lazy_group) => {
                    lazy_group.read(|status| match status {
                        lazy_group::dynamic::View::Initializing(status) =>
                            fused::MainLeafData::RealLeaf(LeafData::LazyGroupInitializing{ message: status.msg.clone(), progress: status.progress }),
                        lazy_group::dynamic::View::Finished(final_group) =>
                            fused::MainLeafData::Subgroup(final_group.as_group_ref()
                                .mark_root()
                                .map_group_data(|group|
                                    if group.group_data().is_root {
                                        GroupData::LazySubgroupRoot
                                    } else { GroupData::LazySubgroupMember }
                                )
                                .map_leaf_data(|l| LeafData::File(l.leaf_data()))
                            ),
                        lazy_group::dynamic::View::Terminated =>
                            fused::MainLeafData::RealLeaf(LeafData::LazyGroupInitializing{ message: None, progress: None })
                    })
                }
                InnerLeafData::RealLeaf(file_data) => {
                    fused::MainLeafData::RealLeaf(LeafData::File(file_data.to_owned())) // OPT: Possibly needless clone
                }
            })
            .fuse()
            .map_structure_error(|err, _| StructureErr::Error(err))
    }
}

/// 'tem is reference to the [TemplateHiearchy] this is part of
pub enum GroupData {
    Static,
    /// Not a real folder to be actually applied, this represents just a rule how to
    /// to organize files inside some folder. There may be multiple of these corresponding to a single folder.
    LazySubgroupRoot,
    LazySubgroupMember
}

#[derive(Clone, Debug)]
pub struct NodeData {
    pub name: Option<String>,
}
#[derive(Clone, Debug)]
pub struct FileData {
    pub path: FileRef,
}

pub enum LeafData {
    File(FileData),
    LazyGroupInitializing {
        message: Option<String>,
        progress: Option<u16>
    }
}
