//! Code related to hiearchy.

use super::*;
pub mod template;
pub use gui::show_hiearchy;
pub use template::TemplateHiearchy;

pub type FileTime = chrono::DateTime<chrono::FixedOffset>;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub pos: Option<backend::geo_lib::Point>,
    pub date: Option<FileTime>,
}

impl FileInfo {
    pub fn transform_for_sorting(
        self,
    ) -> Option<(backend::geo_lib::Point, (String, PathBuf, FileTime))> {
        self.pos
            .and_then(|pos| self.date.map(|date| (pos, (self.name, self.path, date))))
    }

    pub fn transform_after_sorting(
        (pos, (name, path, date)): (backend::geo_lib::Point, (String, PathBuf, FileTime)),
    ) -> Self {
        Self {
            name,
            path,
            pos: Some(pos),
            date: Some(date),
        }
    }
}
