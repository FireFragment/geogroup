use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    convert::Infallible,
    num::NonZero,
    path::{Path, PathBuf},
    sync::{
        atomic::{self, AtomicU16, AtomicU8, AtomicUsize},
        LazyLock, Mutex, RwLock,
    },
    thread,
};

use super::*;

#[derive(Debug, Clone)]
pub enum TResult {
    CreationInProgress,
    Error(Infallible),
    Thumbnail(PathBuf),
}

/// This doesn't actually create thumbnail on Windows yet, thumbnailing only implemented for Linux right now.
pub fn get(path: &Path) -> TResult {
    TResult::Thumbnail(path.to_owned())
}
