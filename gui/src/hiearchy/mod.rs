//! A state of the hiearchy represented as an abstract template. See [`TemplateHiearchy`].
use super::*;

/// A state of the hiearchy.
/// It doesn't represent a concrete hiearchy, but rather abstract template for constructing such a hiearchy.
/// This is how hiearchy is represented in RAM and saved in projects.
///
/// For example, it contains all the "locked" groups' contents, but the automatically sorted ones are
/// represented just by configuration of the sorting algorithm.
///
/// Storing it this way has numerous advantages over storing the entire hiearchy:
///     - More closely represents the intended mental model of using the program, that is, if
///       a group is said to be sorted automatically, its contents is not a fixed hiearchy,
///       but a dynamic one, automatically updated by an algorithm.
///     - Allows for changing the list of files to automatically include the in the hiearchy.
///     - Reading specific parts of the hiearchy without processing its entire contents.
///     - Lower RAM usage.
///
/// If you are interested in examining the actual structure of hiearchy, see [`hiearchy::lazy`]
pub type TemplateHiearchy = backend::fs_hierarchy::FolderRef; // TODO
