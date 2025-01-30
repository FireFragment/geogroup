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
    pub fn from_env_key(cache: PathBuf) -> Self {
        let mut ret =
            Self {
                cache,
                opencage: Opencage::new(std::env::var("OPENCAGE_API_KEY").expect(
                    "Please set OpenCage API key as en environment variable OPENCAGE_API_KEY",
                )),
            };

        ret.opencage.parameters.limit = Some("1");
        ret.opencage.parameters.language = Some("native");
        ret
    }

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
        ret.opencage.parameters.language = Some("native");
        ret
    }

    pub fn reverse_geocode(
        &self,
        point: &Point,
    ) -> Result<HashMap<String, String>, GeocodingError> {
        let mut cache = self.cache.to_owned();
        cache.push(format!("{}_{}", point.0.x, point.0.y));

        if let Some(ret) = fs::read(&cache)
            .ok()
            .map(|cache_contents| Some(bincode::deserialize(&cache_contents[..]).ok()))
            .flatten()
            .flatten()
        {
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

pub fn reverse_geocode_nocache(
    opencage: &Opencage,
    point: &Point,
) -> Result<HashMap<String, String>, GeocodingError> {
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
    /*Useful for debugging formatting */
    /*place
    .iter()
    .filter_map(|(key, value)| value.as_ref().map(|value| format!("{key}: {value}")))
    .collect::<Vec<_>>()
    .join("\n")*/

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
