use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub mod caching_file_item;
pub use caching_file_item::CachedFileSortableItem;
pub mod fs_hierarchy;
pub mod item_source;
pub mod main_hierarchy;
pub mod progress;
pub use geogroup_common::*;
pub use gg_prelude::*;

/// Prelude to use in geogroup-related crates
pub mod gg_prelude {
    use super::*;

    pub use geo as geo_lib;
    pub use geogroup_algo as algorithm;
    pub use geogroup_loaders as loaders;
    pub use geogroup_naming as naming;
    //pub use geogroup_apply as apply;

    pub use lazy_hierarchy;
    pub use lazy_hierarchy::prelude::*;
    pub use loaders::DataLoader as _;
    pub use progress::ProgressCallback;
    pub use progress::TerminatableIterator as _;

    pub use chrono::DateTime;
    pub use either::Either;
    pub use itertools::Itertools;
    pub use thiserror::Error;
    pub use walkdir::WalkDir;
}
