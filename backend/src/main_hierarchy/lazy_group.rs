use super::*;

pub use dynamic::Dynamic;
pub mod dynamic;

/// LazyGroup which is fully "processed", ie. it could be used quickly immediately
#[derive(Debug, From)]
pub enum Final {
    Fs(fs_hierarchy::FolderRef),
    Sorted(LGSorted),
}

#[derive(Debug, Clone)]
/// Simple struct serving as a template for creating [lazy_group::Final]
pub enum Template {
    Fs(fs_hierarchy::FolderRef),
    Sorted {
        item_source: ItemSource,
        params: algorithm::Params,
    },
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
                        manual_name: None,
                        auto_name: AutoNameStatus::named_or_unexpected(
                            node.node_data()
                                .file_name()
                                .map(|name| (*name.to_string_lossy()).to_owned()),
                        ),
                        local_id_path: None,
                        additional_problems: Vec::new(), /*.unwrap_or_else(|| {
                                                             log::error!("Path ending in `..`: {:?}", node.node_data());
                                                             String::new()
                                                         }),*/
                        static_id: None,
                    }),
            ),
            lazy_group::Final::Sorted(sorted, ..) => {
                Either::Right({
                    let global_naming_running = sorted.naming_thread_interface.is_naming_running();
                    sorted
                    .sorter
                    .hierarchy()
                    .map_structure_error(|err, _| match err {})
                    .mark_root()
                    .map_node_data(move |node| {
                        let is_root = if let lazy_hierarchy::NodeRef::Group(ref group) = node {
                            group.group_data().is_root
                        } else { false };

                        let fmt = chrono::format::strftime::StrftimeItems::new_lenient(&sorted.names_time_style);
                        let time = node.node_data().fl_time.first.format_with_items(fmt);
                        NodeData {
                            static_id: Some(node.node_data().id),
                            manual_name: node.node_data().manual_name,
                            auto_name: match node.node_data().auto_name {
                                // TODO: Add date as name
                                algorithm::NameStatus::Named(items) => {
                                    let location_name = items.iter()
                                        .take_while_inclusive(|item|
                                            !sorted.is_name_item_prioritized(&item.name)
                                        )
                                        .filter(|item| !sorted.is_name_item_banned(&item.name))
                                        .map(|item| &item.name)
                                        .join(", ");
                                    AutoNameStatus::Named {
                                        final_name: format!("{time}{location_name}"),
                                        naming_items: Some(items),
                                    }
                                }
                                algorithm::NameStatus::InProgress if global_naming_running => AutoNameStatus::InProgress(Some(time.to_string())),
                                algorithm::NameStatus::InProgress => {
                                    AutoNameStatus::Error {
                                        err: if let Some(ref global_naming_err) = *sorted.naming_thread_interface.get_naming_error() {
                                                NameError::GlobalNamingError(global_naming_err.to_string())
                                            } else {
                                                NameError::UnexpectedError("naming not running but also no global error reported".into())
                                            },
                                        name: Some(time.to_string())
                                    }
                                }
                                algorithm::NameStatus::LocationMissing => {
                                    AutoNameStatus::new_location_missing(Some(time.to_string()))
                                }
                                algorithm::NameStatus::OtherError(err) => AutoNameStatus::Error {
                                    err: NameError::Other(err),
                                    name: Some(time.to_string()),
                                },
                            },
                            /*match node {
                                lazy_hierarchy::NodeRef::Group(gr) =>
                                    gr.group_data().separation.map(|s| format!("Separation: {}m", s/ONE_METER_DISTANCE)),
                                lazy_hierarchy::NodeRef::Leaf(_) => None,
                            } */
                            //Some(format!("{}", node.node_data().local_id_path.into_iter().map(|id| id as u8).join("/"))),
                            local_id_path: Some(
                                node.node_data()
                                    .local_id_path
                                    .into_iter()
                                    .map(|id| id as u8)
                                    .collect(),
                            ),
                            additional_problems:
                                if let lazy_hierarchy::NodeRef::Leaf(leaf) = node
                                    && let Err(e) = &leaf.leaf_data().position
                                {
                                    vec![e.clone().into()]
                                } /*else if // In case of global naming error...
                                    is_root
                                        && !global_naming_running
                                        && let Some(global_naming_error) = *sorted.naming_thread_interface.get_naming_error()
                                {
                                    vec![NodeProblem::GlobalNamingError(global_naming_error.to_string())]
                                }*/ else {
                                    Vec::new()
                                }, // TODO: Report problems with location
                        }
                    })
                    .map_leaf_data(|leaf| leaf.leaf_data().data.to_owned()) // OPT: Possibly needless clone
                    .map_group_data(|_| ())
                })
            }
        }
    }
}

impl Template {
    /// Potentially long-running. Returns [None] on termination
    pub fn into_final(self, progress_callback: &mut ProgressCallback) -> Option<lazy_group::Final> {
        match self {
            Template::Fs(it) => Some(Final::Fs(it)),
            Template::Sorted {
                item_source,
                params,
            } => Some(LGSorted::new(item_source, params, progress_callback)?.into()),
        }
    }
}
