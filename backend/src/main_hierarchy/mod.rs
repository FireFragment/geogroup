use derive_more::From;
use geogroup_algo::geogroup::ManualRename;
use geogroup_loaders::MutDataLoader as _;
use lazy_hierarchy::{concrete::Leaf, GroupRef};
use rayon::iter::ParallelBridge as _;
use core::fmt;
use std::{collections::HashSet, convert::Infallible, hash::Hash, ops::Deref, sync::{atomic::{self, AtomicBool}, Mutex, MutexGuard, OnceLock}, thread};

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
                            auto_name: AutoNameStatus::Named(String::from("Root")), // TODO: Translate
                            local_id_path: None,
                            additional_problems: Vec::new(),
                            static_id: None,
                            manual_name: None
                        },
                    ),
                )],
                (),
                NodeData {
                    auto_name: AutoNameStatus::Named(String::from("Root")), // TODO: Translate
                    local_id_path: None,
                    additional_problems: Vec::new(),
                    static_id: None,
                    manual_name: None
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

/// [Lazy group](lazy_group::Final) (hence the acronym `LG`) which uses the geogroup algorithm to sort its contents
#[derive(Debug)]
pub struct LGSorted {
    item_source: ItemSource,
    sorter: algorithm::Sorter<
        ConcreteSortableItem<
            geo_lib::Point,
            DateTime<chrono::FixedOffset>,
            FileData,
            geogroup_loaders::general_loader::LocationError,
        >,
        nametiles_reader::NamePart,
        geogroup_algo::deep_sorter::naming::NamingErr<NamingLeafErr>,
    >,
    naming_thread_interface: Arc<NamingInterface>
}

/// Interface with the naming thread
#[derive(Debug)]
struct NamingInterface {
    /// Set by the
    naming_error: Mutex<Option<GlobalNamingError>>,
    /// If naming is currently running
    naming_running: AtomicBool
}

impl NamingInterface {
    /// Should be called before starting the naming process.
    /// Checks whether naming is already running. If not, sets the [`naming_running`](Self::naming_running) flag to true and resets the error.
    ///
    /// Returns whether the naming should be started.
    /// (It shouldn't if naming is already running)
    fn pre_start_naming(&self) -> bool {
        // We can reset the error even when naming is already running - it should be reset anyway in that case.
        *self.naming_error.lock().unwrap() = None;

        let already_running = self.naming_running.swap(true, atomic::Ordering::SeqCst);
        !already_running
    }

    /// Should be called after the naming process failed globally by the naming thread.
    /// Resets the [`naming_running`](Self::naming_running) flag to false.
    fn finish_naming_with_error(&self, error: GlobalNamingError) {
        /*let res = self.naming_error.set(error);
        debug_assert_eq!(res.is_err(), false);*/
        *self.naming_error.lock().unwrap() = Some(error);
        self.naming_running.store(false, atomic::Ordering::SeqCst);
    }

    fn get_naming_error(&self) -> impl Deref<Target = Option<GlobalNamingError>> {
        self.naming_error.lock().unwrap()
    }

    fn is_naming_running(&self) -> bool {
        self.naming_running.load(atomic::Ordering::SeqCst)
    }
}

/// Ways the naming can fail _globally_ - that is, no items could be named at all.
/// This is eg. when there's an error with connection to an S3 bucket.
#[derive(Debug, Error)]
pub enum GlobalNamingError {
    #[error("failed to connect to online nametiles database: {0}")]
    BucketError(#[from] nametiles_reader::NewFromBuckerError),
    #[error("unknown tokio error: {0}")]
    TokioError(#[from] tokio::io::Error),
}

#[derive(Error, Debug, Clone)]
pub enum NamingLeafErr {
    #[error("nametiles-related error: {0}")]
    NametilesErr(#[from] nametiles_reader::SimpleError),

    #[error("failed to get photo's location: {0}")]
    LocationErr(#[from] geogroup_loaders::general_loader::LocationError),
}

impl LGSorted {
    /// Potentially long-running. Returns [None] if terminated by progress_callback
    ///
    /// For naming, requires to be run inside of tokio runtime. Naming starts on background.
    pub fn new(
        item_source: ItemSource,
        params: algorithm::Params,
        progress_callback: &mut ProgressCallback,
    ) -> Option<Self> {
        let items: Vec<_> = item_source.clone()
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
            .map_init(||loaders::GeneralLoader::default(), move |loader, path| {
                let loc_data_result = loader.get_data_mut(path.as_ref());
                if let Err(err) = &loc_data_result {
                    log::error!("Failed to read metadata of file {}:\n{err}\nDeveloper info:\n  {err:?}\n  When calling `get_data`", path.to_string_lossy());
                }
                let loc_data = loc_data_result.ok()?; // TODO: Don't ignore errors

                let si_result = loc_data.into_sortable_item(FileData { path: path.clone() });
                if let Err(err) = &si_result {
                    log::error!("Failed to read metadata of file {}:\n{err}\nDeveloper info:\n  {err:?}\n  When converting to SortableItem", path.to_string_lossy());
                }
                si_result.ok() // TODO: Don't ignore errors
            })
            .flatten()
            .collect();

        log::debug!("LGSorted: Loaded all {} files", items.len());
        let should_terminate = progress_callback.call(None, Some("Sorting"));
        if should_terminate {
            return None;
        }
        let sorter = algorithm::Sorter::new(items, params);
        // Useful for debugging until added to the main application
        // sorter.debug_with_fmt_leafs(|l| l.data.path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default());

        let mut res = Self {
            // TODO: Add progress callback
            sorter, // TODO: Don't ignore errors
            item_source,
            naming_thread_interface: Arc::new(NamingInterface {
                naming_error: Mutex::new(None),
                naming_running: AtomicBool::new(false)
            })
        };

        res.start_naming();
        Some(res)
    }

    pub fn params_mut(&mut self) -> &mut algorithm::Params {
        self.sorter.params_mut()
    }

    /// Finishes immediately, starts naming process.
    ///
    /// Returns boolean whether the naming process was actually started, false if it was already running.
    pub fn start_naming(&mut self) {

        let naming_thread_interface = self.naming_thread_interface.clone();
        if !naming_thread_interface.pre_start_naming() {
            return;
        }

        let rt_res = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build();

        let rt = match rt_res {
            Ok(rt) => rt,
            Err(err) => {
                naming_thread_interface.finish_naming_with_error(err.into());
                return;
            }
        };

        {
            let ds = self.sorter.deep_sorter_rc();
            //thread::spawn(|| {
            //let rt_guard = rt.enter();

            thread::spawn(move || {
                let join_handle = rt.block_on(async move {
                    // TODO: Don't block an entire thread
                    // Construct a local task set that can run `!Send` futures.
                    let local = tokio::task::LocalSet::new();

                    // Run the local task set.
                    local.spawn_local(async move {
                        // TODO: Why can't I just spawn it normally?
                        let reader_res = nametiles_reader::NametilesConnection::new_from_env()
                            .await;

                        let reader = match reader_res {
                            Ok(reader) => reader,
                            Err(err) => {
                                naming_thread_interface.finish_naming_with_error(err.into());
                                return;
                            }
                        };

                        log::debug!("LGSorted: Starting naming");

                        //ds.try_naming(async move |_| todo!()).await;
                        ds.try_naming(
                            async move |item| {
                                reader
                                    .get_name(geo::Coord::from(
                                        *item.position.as_ref().map_err(|p| p.clone())?,
                                    ))
                                    .await
                                    .map(|it| HashSet::from_iter(it.into_iter()))
                                    .map_err(|e| {
                                        NamingLeafErr::from(nametiles_reader::SimpleError::from(e))
                                    })
                            },
                            |a, b| a.area.cmp(&b.area),
                        )
                        .await;
                    });

                    local.await;
                });
            });

            //});
        }
    }

    pub fn sorter_mut(&mut self) -> &mut algorithm::Sorter<
            ConcreteSortableItem<
                geo_lib::Point,
                DateTime<chrono::FixedOffset>,
                FileData,
                geogroup_loaders::general_loader::LocationError,
            >,
            nametiles_reader::NamePart,
            geogroup_algo::deep_sorter::naming::NamingErr<NamingLeafErr>,
        > {
        &mut self.sorter
    }
}


#[derive(Debug, Clone)]
pub enum StructureErr<E: std::error::Error> {
    Pending,
    Error(E),
}


/// "Trait alias" to the return value of [`TemplateHiearchy::root_final`] - implements [`lazy_hierarchy::GroupRef`]
pub trait ImplGroupRef: lazy_hierarchy::GroupRef<
    GroupData = GroupData,
    NodeData = NodeData,
    LeafData = LeafData,
    StructureErr = StructureErr<Self::E>,
> {
    type E: std::error::Error;
}

impl<E: std::error::Error, T: lazy_hierarchy::GroupRef<
    GroupData = GroupData,
    NodeData = NodeData,
    LeafData = LeafData,
    StructureErr = StructureErr<E>,
>> ImplGroupRef for T {
    type E = E;
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
    ) -> impl ImplGroupRef<E = std::io::Error> + 'a {
        use lazy_hierarchy::fused;

        self.0
            .root()
            .map_structure_error(|err, _| match err {})
            .map_group_data(|_| GroupData::Static)
            .map_node_data(|n| n.node_data().to_owned()) // OPT: Possibly needless clone
            .map_leaf_data(|leaf| match leaf.leaf_data() {
                InnerLeafData::LazySubgroup(lazy_group) => lazy_group.read(|status| match status {
                    lazy_group::dynamic::View::Initializing(status) => {
                        fused::MainLeafData::RealLeaf(LeafData::LazyGroupInitializing {
                            message: status.msg.clone(),
                            progress: status.progress,
                        })
                    }
                    lazy_group::dynamic::View::Finished(final_group) => {
                        fused::MainLeafData::Subgroup(
                            final_group
                                .as_group_ref()
                                .mark_root()
                                .map_group_data(|group| {
                                    if group.group_data().is_root {
                                        GroupData::LazySubgroupRoot
                                    } else {
                                        GroupData::LazySubgroupMember
                                    }
                                })
                                .map_leaf_data(|l| LeafData::File(l.leaf_data())),
                        )
                    }
                    lazy_group::dynamic::View::Terminated => {
                        fused::MainLeafData::RealLeaf(LeafData::LazyGroupInitializing {
                            message: None,
                            progress: None,
                        })
                    }
                }),
                InnerLeafData::RealLeaf(file_data) => {
                    fused::MainLeafData::RealLeaf(LeafData::File(file_data.to_owned()))
                    // OPT: Possibly needless clone
                }
            })
            .fuse()
            .map_structure_error(|err, _| StructureErr::Error(err))
    }

    pub fn root_for_fs(&self) -> impl apply::LZForApply<Err = RootForRsError<std::io::Error>> {
        self.root_final()
            .map_leaf_data(|leaf| match leaf.leaf_data() {
                LeafData::File(file_data) => {
                    Ok(apply::LeafData {
                        original_path: file_data.path.deref().clone(),
                    })
                }
                LeafData::LazyGroupInitializing { message, progress:_ } => {
                    Err(RootForRsError::LoadingTemplate(message))
                }
            })
            .map_node_data(|node|
                Ok(apply::NodeData {
                    target_name: node.node_data().get_name().unwrap_or("Unnamed".into()).to_owned(),
                }),
            )
            .map_structure_error(|err, _| RootForRsError::StructureError(err))
    }
}

#[derive(Error, Debug, Clone)]
pub enum RootForRsError<SE: std::error::Error> {
    LoadingTemplate(Option<String>),
    NamingInProgress,
    StructureError(StructureErr<SE>),
}

impl<SE: std::error::Error> std::fmt::Display for RootForRsError<SE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RootForRsError::LoadingTemplate(Some(message)) =>
                write!(f, "sorting is in progress (currently, the following happens: {})", message),
            RootForRsError::LoadingTemplate(None) =>
                write!(f, "sorting is in progress"),
            RootForRsError::NamingInProgress =>
                write!(f, "naming is in progress"),
            RootForRsError::StructureError(StructureErr::Pending) =>
                write!(f, "unknown error"), // TODO: Find out what even is StructureErr::Pending
            RootForRsError::StructureError(StructureErr::Error(err)) =>
                write!(f, "{}", err),
        }
    }
}

/// 'tem is reference to the [TemplateHiearchy] this is part of
pub enum GroupData {
    Static,
    /// Not a real folder to be actually applied, this represents just a rule how to
    /// to organize files inside some folder. There may be multiple of these corresponding to a single folder.
    LazySubgroupRoot,
    LazySubgroupMember,
}

#[derive(Clone, Debug, Error)]
pub enum NodeProblem {
    #[error("failed to get location: {0}")]
    NoLocation(#[from] loaders::general_loader::LocationError), // TODO: Make this a reference

    /// String created by formatting [`GlobalNamingError`]
    #[error("cannot set up naming: {0}")]
    GlobalNamingError(String)
}

/// State of automatic naming of the item.
#[derive(Clone, Debug)]
pub enum AutoNameStatus {
    Named(String),
    /// Naming is in progress. Possibly contains partial name (eg. containing only date)
    InProgress(Option<String>),
    /// Naming has been attempted, but failed. Possibly contains partial name (eg. containing only date)
    Error {err: NameError, name: Option<String>},
}

#[derive(Clone, Debug, Error)]
pub enum NameError {
    #[error("location is missing")]
    LocationMissing,
    #[error(transparent)]
    Other(algorithm::deep_sorter::naming::NamingErr<NamingLeafErr>), // TODO: NamingLeafErr::LocationErr is duplicate??

    /// String created by formatting [`GlobalNamingError`]
    #[error("cannot set up naming: {0}")]
    GlobalNamingError(String),
    /// This should never happen, but here we go...
    #[error("unexpected error - this is a bug ({0})")]
    UnexpectedError(String)
}

impl AutoNameStatus {
    pub fn new_location_missing(alt_name: Option<String>) -> Self {
        AutoNameStatus::Error{ err: NameError::LocationMissing, name: alt_name }
    }
    pub fn new_unexpected_err() -> Self {
        AutoNameStatus::Error{ err: NameError::UnexpectedError("unknown".into()), name: None }
    }

    /// Returns `true` if the name status is [`InProgress`].
    ///
    /// [`InProgress`]: NameStatus::InProgress
    #[must_use]
    pub fn is_in_progress(&self) -> bool {
        matches!(self, Self::InProgress(..))
    }
}

impl AutoNameStatus {
    /// Gets name on best-effort basis - if `self` isn't [`NameStatus::Named`], the returned name may be incomplete or missing
    pub fn get_name(&self) -> Option<&str> {
        match self {
            AutoNameStatus::Named(name) => Some(name),
            AutoNameStatus::InProgress(name) => name.as_ref().map(|s| s.as_str()),
            AutoNameStatus::Error { err: _, name } => name.as_ref().map(|s| s.as_str()),
        }
    }

    /// If [None], returns [`NameStatus::UnexpectedError`]
    pub fn named_or_unexpected(input: Option<String>) -> Self {
        match input {
            Some(n) => AutoNameStatus::Named(n),
            None => AutoNameStatus::new_unexpected_err(),
        }
    }
}

// TODO: Allow holding references to the original hierarchy
#[derive(Clone, Debug)]
pub struct NodeData {
    pub auto_name: AutoNameStatus,
    pub manual_name: Option<ManualRename>,
    pub additional_problems: Vec<NodeProblem>,
    /// If you join all `local_id_path`s of a node's parents up to the first [None],
    /// you get an _identification path_ of the node. It's unique in the subtree of the first node whose `local_id_path` is None.
    /// This _identification path_ is preserved during algorithm parameter changes, so it can be used to track selection,
    /// animating the nodes etc.
    pub local_id_path: Option<Vec<u8>>,


    /// ID of the node unique in the lazy subtree.
    pub static_id: Option<u64>,
}

impl NodeData {
    /// Gets name on best-effort basis - the returned name may be incomplete or missing in case of errors
    /// or incomplete naming
    pub fn get_name(&self) -> Option<&str> {
        match self.manual_name {
            Some(ref name) => Some(&name.name),
            None => self.auto_name.get_name(),
        }
    }
}


#[derive(Clone, Debug)]
pub struct FileData {
    pub path: FileRef,
}

pub enum LeafData {
    File(FileData),
    LazyGroupInitializing {
        message: Option<String>,
        progress: Option<u16>,
    },
}
