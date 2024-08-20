use std::path::Display;

use geogroup_common::*;

pub mod algo;

/// Distance between two [points](Point)
pub type Distance = u64;
pub type Flatness = u32;

pub trait Point {
    fn distance(&self, rhs: &Self) -> Distance;
}

pub struct Config {
    pub flatness: u8,
}

impl Point for u64 {
    fn distance(&self, rhs: &Self) -> Distance {
        self.abs_diff(*rhs)
    }
}
