use std::{env, path::PathBuf};

use geocoding::{Opencage, Point};
use geogroup_naming::*;

fn main() {
    let p = Point::new(2.12870, 41.40139);

    let geocoder = RevGeocoder::from_env();

    for i in 0..100 {
        let res = geocoder.reverse_geocode(&p).unwrap();
        let place = build_place(res.iter().map(|(k, v)| (k.as_str(), v.to_owned())));
        println!("{i} | {}", format_place(place));
    }

    println!("{:?}", geocoder.opencage.remaining_calls());
    // Some(2494)
}
