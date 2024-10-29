use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    hash::Hash,
    io::Write,
    iter,
    path::PathBuf,
};

use geocoding::{GeocodingError, Opencage, Point, Reverse};

fn main() {
    let p = Point::new(2.12870, 41.40139);

    let oc = Opencage::new(
        env::var("OPENCAGE_API_KEY")
            .expect("Please set OpenCage API key as en environment variable OPENCAGE_API_KEY"),
    );
    let cache: PathBuf = env::var("GEOGROUP_CACHE_DIR")
        .expect("Env var GEOGROUP_CACHE_DIR missing")
        .into();

    let res = reverse_geocode(cache, &oc, &p).unwrap();

    // "Carrer de Calatrava, 68, 08017 Barcelona, Spain"
    for (key, val) in res.clone() {
        println!("{key} => {val}");
    }

    println!("\n---------------------------------------\n");

    let mut place = address_formatter::PlaceBuilder::default()
        .build_place(res.iter().map(|(k, v)| (k.as_str(), v.to_owned())));
        //.build_place(vec![("continent", String::from("Europe"))]),

    place[address_formatter::Component::Attention] = None;
    place[address_formatter::Component::Postcode] = None;

    for (key, val) in place.clone() {
        println!("{key} => {val:?}");
    }

    println!("\n---------------------------------------\n");

    let formatted = address_formatter::Formatter::default()
        .format_with_config(
            place,
            address_formatter::Configuration {
                abbreviate: Some(true),
                ..Default::default()
            },
        )
        .unwrap()
        .lines()
        .collect::<Vec<_>>()
        .join(", ");

    println!("{formatted}");

    println!("{:?}", oc.remaining_calls());
    // Some(2494)
}

fn reverse_geocode(
    mut cache: PathBuf,
    opencage: &Opencage,
    point: &Point,
) -> Result<HashMap<String, String>, GeocodingError> {
    cache.push(format!("{}_{}", point.0.x, point.0.y));

    println!(" -- Querying cache...");
    if let Some(ret) = fs::read(&cache)
        .ok()
        .map(|cache_contents| Some(bincode::deserialize(&cache_contents[..]).ok()))
        .flatten()
        .flatten()
    {
        println!(" -- Using cache");
        Ok(ret)
    } else {
        let ret = reverse_geocode_nocache(opencage, point);
        if let Ok(ret) = &ret {
            if let Ok(mut file) = File::create(cache) {
                // TODO: Log the error

                if let Ok(serialized) = bincode::serialize(&ret) {
                    // TODO: Log the error
                    file.write_all(&serialized); // TODO: Log the error
                }
            }
        }

        ret
    }
}

fn reverse_geocode_nocache(
    opencage: &Opencage,
    point: &Point,
) -> Result<HashMap<String, String>, GeocodingError> {
    println!(" -- Querying OpenCage...");
    if let [result, ..] = &opencage.reverse_full(point)?.results[..] {
        Ok(result
            .components
            .iter()
            .filter_map(|(key, value)| Some((key.to_owned(), value.as_str()?.to_owned())))
            .collect())
    } else {
        Err(GeocodingError::Reverse)
    }
}

use address_formatter::Place;

/// Returns [Place] with just fields, where both arguments share the same values
pub fn shared(a: &Place, b: &Place) -> Place {
    a.iter()
        .filter(|(comp, val)| b[*comp] == **val)
        .filter_map(|(c, v)| {
            if let Some(rv) = v {
                Some((c, rv.as_str()))
            } else {
                None
            }
        })
        .into()
}
