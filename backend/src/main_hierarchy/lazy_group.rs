use super::*;

pub use dynamic::Dynamic;
pub mod dynamic;

/// LazyGroup which is fully "processed", ie. it could be used quickly immediately
#[derive(Debug, From)]
pub enum Final {
    Fs(fs_hierarchy::FolderRef),
    Sorted (LGSorted),
}

#[derive(Debug, Clone)]
/// Simple struct serving as a template for creating [lazy_group::Final]
pub enum Template {
    Fs(fs_hierarchy::FolderRef),
    Sorted {
        item_source: ItemSource,
        params: algorithm::Params
    }
}

impl Final {
    pub fn as_group_ref<'a>(
        &'a self,
    ) -> impl GroupRef<
        GroupData = (),
        LeafData = FileData,
        NodeData = NodeData,
        StructureErr = std::io::Error,
    > + 'a {
        match self {
            lazy_group::Final::Fs(folder_ref) => Either::Left(
                folder_ref
                    .to_owned()
                    .map_leaf_data(|leaf| FileData {
                        path: FileRef::new(leaf.node_data()),
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
            lazy_group::Final::Sorted(sorted, ..) => Either::Right(
                sorted.sorter
                    .hierarchy()
                    .map_structure_error(|err, _| match err {})
                    .map_node_data(|_| NodeData { name: None })
                    .map_leaf_data(|leaf| leaf.leaf_data().data.to_owned()) // OPT: Possibly needless clone
                    .map_group_data(|_| ()),
            ),
        }
    }
}


impl Template {
    /// Potentially long-running. Returns [None] on termination
    pub fn into_final(self, progress_callback: ProgressCallback) -> Option<lazy_group::Final> {
        match self {
            Template::Fs(it) => Some(Final::Fs(it)),
            Template::Sorted { item_source, params } => 
                Some(LGSorted::new(item_source, params, progress_callback)?.into()),
        }
    }
}
