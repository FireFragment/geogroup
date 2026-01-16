use lazy_hierarchy::NodeRef;

use super::*;
use std::{error, path::PathBuf};

#[derive(Clone, Debug)]
pub struct FolderRef(PathBuf);
#[derive(Clone, Debug)]
pub struct FileRef(PathBuf);

/// Operation failed because a path is neither a file nor directory.
#[derive(Error, Debug)]
#[error("path {0} is neither a file nor a directory")]
pub struct ErrFileNorDir(PathBuf);

///  - Returns [`NodeRef::Group`] iff `path.is_dir()`
///  - Returns [`NodeRef::Leaf`] iff `path.is_file()`
///  - Returns [`Result::Err`] iff `!path.is_file() && path.is_dir()`
pub fn new(path: PathBuf) -> Result<NodeRef<FolderRef>, ErrFileNorDir> {
    if path.is_file() {
        Ok(NodeRef::Leaf(FileRef(path)))
    } else if path.is_dir() {
        Ok(NodeRef::Group(FolderRef(path)))
    } else {
        Err(ErrFileNorDir(path))
    }
}

impl lazy_hierarchy::GroupRef for FolderRef {
    type NodeData = PathBuf;
    type LeafData = ();
    type GroupData = ();
    type StructureErr = std::io::Error;
    type LeafRef = FileRef;

    fn get_children(
        &self,
    ) -> Result<impl Iterator<Item = lazy_hierarchy::NodeRef<Self>>, Self::StructureErr>
    where
        Self: Sized,
    {
        Ok(self
            .0
            .read_dir()?
            .map(|entry| Ok(new(entry.map_err(Either::Left)?.path()).map_err(Either::Right)?))
            .filter_map(|child: Result<_, Either<std::io::Error, _>>| match child {
                Ok(child) => Some(child),
                Err(err) => {
                    // TODO: Handle fs errors better / show them to user
                    // TODO: Check if this isn't spammy
                    log::error!(
                        "Failed to read child of {}, ignoring it:\n{err}",
                        self.0.display()
                    );
                    None
                }
            }))
    }

    fn group_data(&self) -> Self::GroupData {
        ()
    }

    fn node_data(&self) -> Self::NodeData {
        self.0.clone()
    }
}

impl lazy_hierarchy::LeafRef for FileRef {
    type LeafData = ();

    type NodeData = PathBuf;

    fn leaf_data(&self) -> Self::LeafData {
        ()
    }

    fn node_data(&self) -> Self::NodeData {
        self.0.clone()
    }
}
