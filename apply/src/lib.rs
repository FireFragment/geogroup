use std::{fmt::format, os::unix::fs::chown, path::PathBuf};

use geogroup_common::*;

/// Leaf data of a hiearchy to be applied
pub struct ApplyLeaf {
    /// What should the file be renamed to
    pub target_name: String,
    pub original_path: PathBuf
}

pub fn apply_by_copy(hiearchy: HiearchyItem<ApplyLeaf, String>, target_dir: PathBuf) -> std::io::Result<()>  {
    apply_by_copy_inner(hiearchy, target_dir, 0)
}

fn apply_by_copy_inner(hiearchy: HiearchyItem<ApplyLeaf, String>, target_dir: PathBuf, idx: usize) -> std::io::Result<()>  {
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)?;
    }

    match hiearchy {
        HiearchyItem::Group(children, name) => {
            let mut tg_dir = target_dir.to_owned();
            tg_dir.push(format!("{idx} - {name}"));

            for (idx, child) in children.into_iter().enumerate() {
                apply_by_copy_inner(child, tg_dir.clone(), idx)?;
            }
            Ok(())
        },
        HiearchyItem::Item(leaf) => {
            let mut tg_dir = target_dir.clone();
            tg_dir.push(format!("{idx} - {}{}", leaf.target_name, leaf.original_path.extension().map(|ext| format!(".{}", ext.to_str().unwrap() /* TODO: Don't unwrap */)).unwrap_or_default()));
            println!("Writing {}", tg_dir.to_string_lossy());
            std::fs::copy(leaf.original_path, tg_dir).map(|_| ())
        }
    }
}