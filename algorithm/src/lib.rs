use std::path::Display;

use geogroup_common::*;

pub mod algo;

pub trait Point {
    fn distance(&self, rhs: &Self) -> Distance;
}

pub struct Config {
    pub flatness: u8,
}

impl Point for u64 {
    fn distance(&self, rhs: &Self) -> geogroup_common::Distance {
        self.abs_diff(*rhs)
    }
}
