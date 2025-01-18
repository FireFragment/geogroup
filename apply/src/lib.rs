use std::{os::unix::fs::chown, path::PathBuf};

use geogroup_common::*;

/// Leaf data of a hiearchy to be applied
pub struct ApplyLeaf {
    /// What should the file be renamed to
    pub target_name: String,
    pub original_path: PathBuf
}

pub fn apply_by_copy(hiearchy: HiearchyItem<ApplyLeaf, String>, target_dir: PathBuf) -> std::io::Result<()>  {
    std::fs::create_dir_all(&target_dir)?;

    match hiearchy {
        HiearchyItem::Group(children, name) => {
            for child in children {
                let mut tg_dir = target_dir.to_owned();
                tg_dir.push(&name);
                apply_by_copy(child, tg_dir)?;
            }
            Ok(())
        },
        HiearchyItem::Item(leaf) => {
            let mut tg_dir = target_dir.clone();
            tg_dir.push(leaf.target_name);
            std::fs::copy(leaf.original_path, tg_dir).map(|_| ())
        }
    }
}