//! This crate handles filesystem operations for applying output of Geogroup.

use std::{
    convert::Infallible,
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
};

use lazy_hierarchy::{GroupRef, LeafRef, NodeRef};
use sanitise_file_name::sanitise;
use thiserror::Error;
use unwrap_infallible::UnwrapInfallible;

/// Leaf data of a hiearchy to be applied
pub struct ApplyLeaf {
    /// What should the file be renamed to
    pub target_name: String,
    pub original_path: PathBuf,
}

pub struct NodeData {
    /// How to name the node, without its extension
    /// Will be sanitised
    pub target_name: String,
}

pub struct LeafData {
    /// Where is the original file
    pub original_path: PathBuf,
}

/// Automatically implemented trait for a lazy hierarchy that can be applied to filesystem
pub trait LZForApply:
    GroupRef<
    NodeData = Result<NodeData, Self::Err>,
    LeafData = Result<LeafData, Self::Err>,
    StructureErr = Self::Err,
>
{
    type Err;
}
impl<T, E> LZForApply for T
where
    T: GroupRef<NodeData = Result<NodeData, E>, LeafData = Result<LeafData, E>, StructureErr = E>,
{
    type Err = E;
}

#[derive(Error, Debug)]
pub enum ApplyError<E> {
    #[error("transferring file {0} to {1} failed: {2}")]
    FileIoError(PathBuf, PathBuf, std::io::Error),
    #[error("creating directory {0} failed: {1}")]
    MkDirError(PathBuf, std::io::Error),
    #[error(transparent)]
    HierarchyError(#[from] E),
}

/// Function `report_progress` takes the original path and the target path as arguments
/// Function `transfer_file` is only applied on files
pub fn apply_by<E>(
    hiearchy: impl LZForApply<Err = E>,
    target_dir: &Path,
    mut transfer_file: impl FnMut(&Path, &Path) -> std::io::Result<()> + Clone,
) -> Result<(), ApplyError<E>> {
    for child in hiearchy.get_children()? {
        let target_name = sanitise(&child.node_data()?.target_name);
        let extension = if let NodeRef::Leaf(ref leaf) = child {
            leaf.leaf_data()?
                .original_path
                .extension()
                .map(|ext| format!(".{}", ext.display()))
                .unwrap_or_default()
                .to_owned()
        } else {
            String::new()
        };

        let mut target_path = target_dir.join(format!("{target_name}{extension}"));

        let mut idx = 0;
        while target_path.exists() {
            target_path = target_dir.join(format!("{target_name}_{idx:0>4}{extension}"));
            idx += 1;
        }

        match child {
            NodeRef::Group(group) => {
                fs::create_dir_all(&target_path)
                    .map_err(|e| ApplyError::MkDirError(target_path.clone(), e))?;
                apply_by(group, &target_path, transfer_file.clone())?;
            }
            NodeRef::Leaf(leaf) => {
                let original_path = leaf.leaf_data()?.original_path;
                transfer_file(&original_path, &target_path)
                    .map_err(|e| ApplyError::FileIoError(original_path, target_path, e))?;
            }
        }
    }
    Ok(())
}

pub fn apply_by_copy<E>(
    hiearchy: impl LZForApply<Err = E>,
    target_dir: &Path,
    mut report_progress: impl FnMut(&Path, &Path) + std::clone::Clone,
) -> Result<(), ApplyError<E>> {
    apply_by(hiearchy, target_dir, move |a, b| {
        report_progress(a, b);
        fs::copy(a, b)?;
        Ok(())
    })
}

pub fn apply_by_symlink<E>(
    hiearchy: impl LZForApply<Err = E>,
    target_dir: &Path,
    mut report_progress: impl FnMut(&Path, &Path) + Clone,
) -> Result<(), ApplyError<E>> {
    #[cfg(unix)]
    apply_by(hiearchy, target_dir, move |a, b| {
        report_progress(a, b);
        symlink_file(a, b)
    })
}

/// Works on windows and unix
pub fn symlink_file(a: &Path, b: &Path) -> io::Result<()> {
    #[cfg(unix)]
    std::os::unix::fs::symlink(a, b)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(a, b)?;

    Ok(())
}
