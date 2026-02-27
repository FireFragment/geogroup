use std::{
    cmp::min, collections::{HashMap, HashSet}, num::NonZero, path::{Path, PathBuf}, sync::{
        atomic::{self, AtomicU16, AtomicU8, AtomicUsize},
        LazyLock, Mutex, RwLock,
    }, thread
};

use allmytoes::{AMTConfiguration, ToeErrorType};

use super::*;

#[derive(Debug, Clone)]
pub enum TResult {
    CreationInProgress,
    Error(ToeErrorType),
    Thumbnail(PathBuf),
}

/// **Note:** sometimes might ignore a request to create thumbnail and return [`TResult::CreationInProgressOrFailed`]
/// if too many thumbnails are already being created.
pub fn get(path: &Path) -> TResult {
    // TODO: Examine memory usage of this function - make sure it isn't memory hungry by storing
    // paths to the thumbnails _forever_
    static THUMBNAILER: LazyLock<allmytoes::AMT> =
        LazyLock::new(|| allmytoes::AMT::new(&AMTConfiguration::default()));
    static THUMBNAILS: LazyLock<Mutex<HashMap<PathBuf, TResult>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    static CURRENTLY_RUNNING_THUMNAILING_JOBS: AtomicUsize = AtomicUsize::new(0);
    const MAX_THUMBNAILING_JOBS: LazyLock<NonZero<usize>> = LazyLock::new(||
        // We don't want to create too many thumbnails if
        // the user quickly scrolls through a long list of images
        // so we limit the maximum number of thumbnails creation jobs
        // to 20.
        min(
            thread::available_parallelism()
                .unwrap_or(NonZero::new(8).unwrap()),
            NonZero::new(20).unwrap()
        ));

    let mut thumbnails = THUMBNAILS.lock().unwrap();

    if let Some(res) = thumbnails.get(path) {
        return res.to_owned();
    } else {
        // Make sure there aren't too many thumbnailing jobs already running...
        if CURRENTLY_RUNNING_THUMNAILING_JOBS.load(atomic::Ordering::Relaxed)
            >= usize::from(*MAX_THUMBNAILING_JOBS)
        {
            return TResult::CreationInProgress; // TODO: We can return something to indicate
                                                // that thumbnail is not being created
        }

        // Start a new job for thumnailing this path
        let path = path.to_owned();
        thumbnails.insert(path.clone(), TResult::CreationInProgress);
        drop(thumbnails); // Unlock the mutex

        // TODO: What does `Relaxed` mean here?
        CURRENTLY_RUNNING_THUMNAILING_JOBS.fetch_add(1, atomic::Ordering::Relaxed);
        thread::spawn(move || {
            let thumb = THUMBNAILER.get(&path, allmytoes::ThumbSize::Normal);

            THUMBNAILS
                .lock()
                .expect("Already held by this thread??")
                .insert(
                    path.clone(),
                    match thumb {
                        Ok(t) => TResult::Thumbnail(t.path.into()),
                        Err(err) => TResult::Error(err),
                    },
                );

            log::trace!("Thumbnail has been retrieved");

            // TODO: What does `Relaxed` mean here?
            CURRENTLY_RUNNING_THUMNAILING_JOBS.fetch_sub(1, atomic::Ordering::Relaxed);
        });

        TResult::CreationInProgress
    }
}
