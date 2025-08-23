use std::sync::OnceLock;

use super::*;

pub type Item = PathBuf;

/// Unlike [`ItemSourceTemplate`] performs caching
pub struct ItemSource(ItemSourceInner);

pub enum ItemSourceTemplate {
    Enumerated(Vec<PathBuf>),
    Complex(ComplexItemSourceTemplate),
}

impl From<ItemSourceTemplate> for ItemSource {
    fn from(value: ItemSourceTemplate) -> Self {
        match value {
            ItemSourceTemplate::Enumerated(paths) => Self(ItemSourceInner::Enumerated(paths)),
            ItemSourceTemplate::Complex(it) => Self(ItemSourceInner::Complex {
                it,
                cache: OnceLock::new(),
            }),
        }
    }
}

pub struct Cache {
    pub paths: Vec<PathBuf>,
    pub errors: Vec<anyhow::Error>,
}

enum ItemSourceInner {
    /// Complex, cached condition
    Complex {
        it: ComplexItemSourceTemplate,
        cache: OnceLock<Cache>, // TODO: Implement watching for changes in filesystem
    },
    /// Specific enumerated items
    Enumerated(Vec<PathBuf>),
}

/// Item source which is hard to compute and therefore requires caching to be used while maintaining performance
#[derive(Clone)]
pub enum ComplexItemSourceTemplate {
    /// All files in a directory, recursively
    EntireDir(PathBuf),
}

impl ComplexItemSourceTemplate {
    pub fn create_cache(&self, progress_callback: ProgressCallback) -> Option<Cache> {
        // TODO: Terminate by progress_callback
        match self {
            ComplexItemSourceTemplate::EntireDir(dir) => {
                let (entries, errors): (Vec<_>, Vec<_>) = WalkDir::new(dir)
                    .into_iter()
                    .with_progress_callback(
                        progress_callback,
                        |entry| {
                            entry
                                .as_ref()
                                .map_or_else(|err| err.path(), |entry| Some(entry.path()))
                                .map(|path| path.to_string_lossy().to_string())
                        },
                        100,
                    )
                    .map(|entry| match entry {
                        Ok(entry) => Ok(entry.into_path()),
                        Err(err) => Err(err.into()),
                    })
                    .partition_result();

                Some(Cache {
                    paths: entries,
                    errors: errors,
                })
            }
        }
    }
}

impl ItemSource {
    /// Whether or not [`Self::into_concrete`] is fast to execute.
    pub fn is_fast(&self) -> bool {
        match &self.0 {
            ItemSourceInner::Enumerated(_) => true,
            ItemSourceInner::Complex { cache, .. } => cache.get().is_some(),
        }
    }

    /// Prepares a cache if it didn't exist. If it already did, this method should be fast.
    ///
    /// Returns [`true`] if it wasn't terminated by the `progress_callback` argument and finished successfully.
    /// In this case, `into_concrete` is garantueed to be fast.
    ///
    /// May be slow. The `progress_callback` is repeatedly called - if it returns [`true`], the function exits early.
    pub fn prepare_cache(&self, progress_callback: ProgressCallback) -> bool {
        match &self.0 {
            ItemSourceInner::Complex { it, cache } => {
                if cache.get().is_none() {
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
    pub fn into_concrete(&self) -> &Vec<PathBuf> {
        match &self.0 {
            ItemSourceInner::Complex { it, cache } => {
                &cache
                    .get_or_init(|| it.create_cache(ProgressCallback::new_ignore())
                    .expect("BUG: Operation was allegedly *somehow* terminated with `ProgressCallback::new_ignore()`"))
                    .paths
            }
            ItemSourceInner::Enumerated(paths) => paths,
        }
    }

    /// Clears the cache, making it empty until initialized with eg. [`Self::prepare_cache`]
    pub fn clear_cache(&mut self) {
        match &mut self.0 {
            ItemSourceInner::Complex { ref mut cache, .. } => {
                cache.take();
            }
            ItemSourceInner::Enumerated(_) => {}
        }
    }
}
