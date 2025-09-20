use std::{
    collections::{HashMap, HashSet},
    sync::{LazyLock, Mutex, RwLock},
    thread,
};

use super::*;


pub enum TResult {
    CreationInProgressOrFailed, // TODO: If failed, return Error instead
    Error(allmytoes::ToeErrorType),
    Thumbnail(allmytoes::Thumb),
}

pub fn get(path: &Path) -> TResult {
    static THUMBNAILER: LazyLock<allmytoes::AMT> =
        LazyLock::new(|| allmytoes::AMT::new(&AMTConfiguration::default()));


    static THUMBNAILING_JOBS: LazyLock<
        Mutex<HashSet<PathBuf>>,
    > = LazyLock::new(|| Mutex::new(HashSet::new()));

    let res = THUMBNAILER.get_without_creation(path, allmytoes::ThumbSize::Normal);

    match res {
        Ok(thumb) => TResult::Thumbnail(thumb),
        Err(allmytoes::ToeErrorType::ThumbDoesNotExistsAndCreationDisallowed) => {
            let mut thumbnailing_jobs = THUMBNAILING_JOBS.lock().unwrap();

            // If there were no attempts at thumnailing this path...
            if !thumbnailing_jobs.contains(path) {
                //... we start a new attempt at thumnailing this path
                let path = path.to_owned();
                thumbnailing_jobs.insert(path.clone());
                thread::spawn(move || {
                    THUMBNAILER.get(&path, allmytoes::ThumbSize::Normal);
                });
            }
            TResult::CreationInProgressOrFailed
        }
        Err(err) => TResult::Error(err),
    }
}
