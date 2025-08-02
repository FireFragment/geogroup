use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub mod fs_hierarchy;
pub use geogroup_common::*;
pub use gg_prelude::*;

/// Prelude to use in geogroup-related crates
pub mod gg_prelude {
    pub use geo as geo_lib;
    pub use geogroup_algo as algorithm;
    pub use geogroup_loaders as loaders;
    pub use geogroup_naming as naming;
    //pub use geogroup_apply as apply;

    pub use lazy_hierarchy;
    pub use lazy_hierarchy::prelude::*;
    pub use loaders::DataLoader as _;

    pub use either::Either;
    pub use itertools::Itertools;
    pub use thiserror::Error;
}
