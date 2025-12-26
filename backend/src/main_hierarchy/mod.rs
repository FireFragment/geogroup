use std::{collections::HashSet, convert::Infallible, hash::Hash, thread};
use derive_more::From;
use lazy_hierarchy::{concrete::Leaf, GroupRef};
use rayon::iter::ParallelBridge as _;

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
                            local_id_path: None,
                        },
                    ),
                )],
                (),
                NodeData {
                    name: Some(String::from("Root")), // TODO: Translate
                    local_id_path: None,
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

/// Lazy group (hence the acronym `LG`) which uses the geogroup algorithm to sort its contents
#[derive(Debug)]
pub struct LGSorted {
    item_source: ItemSource,
    sorter: algorithm::Sorter<
        ConcreteSortableItem<geo_lib::Point, DateTime<chrono::FixedOffset>, FileData, geogroup_loaders::general_loader::LocationError>,
        String,
        geogroup_algo::deep_sorter::naming::NamingErr<NamingLeafErr>
    >,
}

#[derive(Error, Debug)]
pub enum NamingLeafErr {
    #[error("Nametiles-related error: {0}")]
    NametilesErr(#[from] nametiles_reader::Error),

    #[error("Failed to get photo's location: {0}")]
    LocationErr(#[from] geogroup_loaders::general_loader::LocationError),
}

impl LGSorted {
    /// Potentially long-running. Returns [None] if terminated by progress_callback
    ///
    /// For naming, requires to be run inside of tokio runtime
    pub fn new(item_source: ItemSource, params: algorithm::Params, progress_callback: &mut ProgressCallback) -> Option<Self> {
        let items = item_source.clone()
            .into_concrete()
            .with_progress_callback(
                // TODO: Don't show progress from this as this should be much faster than reading metadata of the files
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
                                             // only this way can size hint be provided
            .with_progress_callback(
                progress_callback,
                |entry| {
                    entry
                        .as_ref()
                        .map_or_else(|err| err.path(), |entry| Some(entry))
                        .map(|path| format!("Reading metadata: {}", path.to_string_lossy().to_string()))
                },
                10,
            )
            .par_bridge()
            .filter_map(|entry| {
                if let Err(err) = &entry {
                    log::error!("Failed to read file: {err}");
                }

                entry.ok() // TODO: Don't ignore errors
            })
            .filter_map(|path| {
                let loc_data_result = loaders::GeneralLoader.get_data(path.as_ref());
                if let Err(err) = &loc_data_result {
                    log::error!("Failed to read metadata of file {}:\nWhen calling `get_data`\n{err}\n{err:?}", path.to_string_lossy());
                }
                let loc_data = loc_data_result.ok()?; // TODO: Don't ignore errors

                let si_result = loc_data.into_sortable_item(FileData { path: path.clone() });
                if let Err(err) = &si_result {
                    log::error!("Failed to read metadata of file {}:\nWhen converting to SortableItem\n{err}\n{err:?}", path.to_string_lossy());
                }
                si_result.ok() // TODO: Don't ignore errors
            })
            .collect();

        log::debug!("LGSorted: Loaded all files");
        let should_terminate = progress_callback.call(None, Some("Sorting"));
        if should_terminate {
            return None;
        }
        let sorter = algorithm::Sorter::new(
            items,
            params
        );
        // Useful for debugging until added to the main application
        // sorter.debug_with_fmt_leafs(|l| l.data.path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default());


        let rt = tokio::runtime::Builder::new_current_thread().build().expect("TODO");
        {
            let ds = sorter.deep_sorter_rc();
            //thread::spawn(|| {
            //let rt_guard = rt.enter();

            thread::spawn(move || {
                let join_handle = rt.block_on(async move { // TODO: Don't block
                    // Construct a local task set that can run `!Send` futures.
                    let local = tokio::task::LocalSet::new();

                    // Run the local task set.
                    local.spawn_local(async move { // TODO: Why can't I just spawn it normally?
                        let reader = nametiles_reader::NametilesConnection::new_from_file(
                            &PathBuf::from("/nix/temporary/nametilesgen/out.pmtiles") // TODO: Remove
                        ).await.expect("TODO");

                        //ds.try_naming(async move |_| todo!()).await;
                        ds.try_naming(async move |item| {
                            reader.get_name(geo::Coord::from(*item.position.as_ref().map_err(|p| p.clone())?))
                                .await.map(|it| HashSet::from_iter(it.into_iter()))
                                .map_err(|e| NamingLeafErr::from(e))
                        }).await;
                    });

                    local.await;
                });
            });

            //});

        }

        Some(Self {
            // TODO: Add progress callback
            sorter,  // TODO: Don't ignore errors
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
    /// to be applied, see [GroupData::LazySubgroupRoot] which does not correspond to a folder.
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
    /// If you join all `local_id_path`s of a node's parents up to the first [None],
    /// you get an _identification path_ of the node. It's unique in the subtree of the first node whose `local_id_path` is None.
    /// This _identification path_ is preserved during algorithm parameter changes, so it can be used to track selection,
    /// animating the nodes etc.
    pub local_id_path: Option<Vec<u8>>
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
