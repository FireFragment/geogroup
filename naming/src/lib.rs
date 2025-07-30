pub mod geocoding_utils;
pub use geocoding_utils::*;
//pub mod logic;

pub use geocoding_utils::RevGeocoder;

pub struct PrefetchProgressReport {
    /// How many places have been fetched
    pub done: usize,

    /// How many places are remaining
    pub total: usize,
}
