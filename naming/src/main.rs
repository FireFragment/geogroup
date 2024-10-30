use std::{env, path::PathBuf};

use geocoding::{Opencage, Point};
use geogroup_naming::*;

fn main() {
    let p = Point::new(2.12870, 41.40139);

    let oc = get_opencage("native");
    let cache: PathBuf = env::var("GEOGROUP_CACHE_DIR")
        .expect("Env var GEOGROUP_CACHE_DIR missing")
        .into();

    for i in 0..100 {
        let res = reverse_geocode(cache.clone(), &oc, &p).unwrap();
        let place = build_place(res.iter().map(|(k, v)| (k.as_str(), v.to_owned())));
        println!("{i} | {}", format_place(place));
    }

    println!("{:?}", oc.remaining_calls());
    // Some(2494)
}
