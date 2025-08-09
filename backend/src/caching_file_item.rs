use super::*;
use std::collections::HashMap;

pub struct CachedFileSortableItem<'cache>(PathBuf, &'cache Cache);

//impl<'cache> SortableItem for CachedFileSortableItem<'cache> {}

pub struct Cache {
    points: HashMap<PathBuf, geo_lib::Point>,
    dates: HashMap<PathBuf, DateTime<chrono::FixedOffset>>,
}
