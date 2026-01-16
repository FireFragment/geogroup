use std::sync::OnceLock;

use super::*;

pub type Item = PathBuf;

/// Unlike [`ItemSource`] performs caching
pub struct CachingItemSource(ItemSourceInner);

#[derive(Clone, Debug)]
pub enum ItemSource {
    Enumerated(Vec<FileRef>),
    /// All files in a directory, recursively
    EntireDir(FileRef),
}

impl From<ComplexItemSource> for ItemSource {
    fn from(val: ComplexItemSource) -> Self { match val {
        ComplexItemSource::EntireDir(dir) => ItemSource::EntireDir(dir),
    } }
}

impl From<ItemSource> for CachingItemSource {
    fn from(value: ItemSource) -> Self {
        match value {
            ItemSource::Enumerated(paths) => Self(ItemSourceInner::Enumerated(paths)),
            ItemSource::EntireDir(dir) => Self(ItemSourceInner::Complex {
                it: ComplexItemSource::EntireDir(dir),
                cache: OnceLock::new(),
            }),
        }
    }
}

pub struct Cache {
    pub paths: Vec<FileRef>,
    pub errors: Vec<anyhow::Error>,
}

enum ItemSourceInner {
    /// Complex, cached condition
    Complex {
        it: ComplexItemSource,
        cache: OnceLock<Cache>, // TODO: Implement watching for changes in filesystem
    },
    /// Specific enumerated items
    Enumerated(Vec<FileRef>),
}

/// Item source which is hard to compute and therefore requires caching to be used while maintaining performance
#[derive(Clone)]
pub enum ComplexItemSource {
    /// All files in a directory, recursively
    EntireDir(FileRef),
}




impl ComplexItemSource {
    pub fn create_cache(&self, _progress_callback: &mut ProgressCallback) -> Option<Cache> {
        todo!()//ItemSource::from(self.to_owned()).into_concrete(progress_callback)
    }
}
impl ItemSource {
    /// Potentially long-running.
    ///
    /// The iterator yields elements at approximately regular intervals.
    /// This can be used for progress reporting (eg. calling `with_progress_callback` on the returned iterator)
    pub fn into_concrete<'a>(
        self,
    ) -> impl Iterator<Item = Result<FileRef, walkdir::Error>> + Send {
        // TODO: Terminate by progress_callback
        match self {
            ItemSource::Enumerated(files) => {
                Either::Left(files.into_iter().map(Ok))
            }
            ItemSource::EntireDir(dir) => {
                Either::Right(
                    WalkDir::new((*dir).clone())
                    .into_iter()
                    .map(|entry| match entry {
                        Ok(entry) => Ok(entry.into_path().into()),
                        Err(err) => Err(err.into()),
                    })
                )
            }
        }
    }
}

impl CachingItemSource {
    /// Whether or not [`Self::into_concrete`] is fast to execute.
    pub fn is_fast(&self) -> bool {
        match &self.0 {
            ItemSourceInner::Enumerated(_) => true,
            ItemSourceInner::Complex { cache, .. } => OnceLock::get(cache).is_some(),
        }
    }

    /// Prepares a cache if it didn't exist. If it already did, this method should be fast.
    ///
    /// Returns [`true`] if it wasn't terminated by the `progress_callback` argument and finished successfully.
    /// In this case, `into_concrete` is garantueed to be fast.
    ///
    /// May be slow. The `progress_callback` is repeatedly called - if it returns [`true`], the function exits early.
    pub fn prepare_cache(&self, progress_callback: &mut ProgressCallback) -> bool {
        match &self.0 {
            ItemSourceInner::Complex { it, cache } => {
                if OnceLock::get(cache).is_none() {
                    if let Some(new_cache) = it.create_cache(progress_callback) {
                        let _ = cache.set(new_cache); // The error is thrown iff cache is already inhabited,
                                                      // but we made sure it isn't by wrapping this in
                                                      // `if cache.get().is_none()`
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            ItemSourceInner::Enumerated(_) => true,
        }
    }

    /// May be slow if cache is missing. To ensure good run time, run [`prepare_cache`] beforehand
    pub fn into_concrete(&self) -> &Vec<FileRef> {
        match &self.0 {
            ItemSourceInner::Complex { it, cache } => {
                &cache
                    .get_or_init(|| it.create_cache(&mut ProgressCallback::new_ignore())
                    .expect("BUG: Operation was allegedly *somehow* terminated with `ProgressCallback::new_ignore()`"))
                    .paths
            }
            ItemSourceInner::Enumerated(paths) => paths,
        }
    }

    /// Clears the cache, making it empty until initialized with eg. [`Self::prepare_cache`]
    pub fn clear_cache(&mut self) {
        match &mut self.0 {
            ItemSourceInner::Complex { cache, .. } => {
                cache.take();
            }
            ItemSourceInner::Enumerated(_) => {}
        }
    }
}
