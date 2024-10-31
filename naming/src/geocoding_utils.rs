//! Utilities for actual geocoding and caching the results

use address_formatter::Place;
use geocoding::{GeocodingError, Opencage, Point};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

pub struct RevGeocoder<'a> {
    pub cache: PathBuf,
    pub opencage: Opencage<'a>,
}

impl RevGeocoder<'_> {
    pub fn from_env() -> Self {
        let mut ret =
            Self {
                cache: std::env::var("GEOGROUP_CACHE_DIR")
                    .expect("Env var GEOGROUP_CACHE_DIR missing")
                    .into(),
                opencage: Opencage::new(std::env::var("OPENCAGE_API_KEY").expect(
                    "Please set OpenCage API key as en environment variable OPENCAGE_API_KEY",
                )),
            };
        ret.opencage.parameters.limit = Some("1");
        ret
    }

    pub fn reverse_geocode(
        &self,
        point: &Point,
    ) -> Result<HashMap<String, String>, GeocodingError> {
        let mut cache = self.cache.to_owned();
        cache.push(format!("{}_{}", point.0.x, point.0.y));

        println!(" -- Querying cache...");
        if let Some(ret) = fs::read(&cache)
            .ok()
            .map(|cache_contents| Some(bincode::deserialize(&cache_contents[..]).ok()))
            .flatten()
            .flatten()
        {
            //println!(" -- Using cache");
            let mut all_geocode_cnt = ALL_GEOCODE_CNT.lock().unwrap();
            println!("All: #{all_geocode_cnt}");
            *all_geocode_cnt += 1;

            Ok(ret)
        } else {
            let ret = reverse_geocode_nocache(&self.opencage, point);
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
}

static REAL_GEOCODE_CNT: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::new(0));
static ALL_GEOCODE_CNT: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::new(0));

pub fn reverse_geocode_nocache(
    opencage: &Opencage,
    point: &Point,
) -> Result<HashMap<String, String>, GeocodingError> {
    let mut real_geocode_cnt = REAL_GEOCODE_CNT.lock().unwrap();
    let mut all_geocode_cnt = ALL_GEOCODE_CNT.lock().unwrap();
    println!("Real: #{real_geocode_cnt} | All: #{all_geocode_cnt}");
    *real_geocode_cnt += 1;
    *all_geocode_cnt += 1;

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

pub fn build_place<'a>(data: impl IntoIterator<Item = (&'a str, String)>) -> Place {
    let mut place = address_formatter::PlaceBuilder::default().build_place(data);

    place[address_formatter::Component::Attention] = None;
    place[address_formatter::Component::Postcode] = None;

    place
}

pub fn format_place(place: Place) -> String {
    address_formatter::FORMATTER
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
        .join(", ")
}
