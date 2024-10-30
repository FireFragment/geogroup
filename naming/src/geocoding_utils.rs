use address_formatter::Place;
use geocoding::{GeocodingError, Opencage, Point};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

pub fn get_opencage<'a>(lang: &'a str) -> Opencage<'a> {
    let mut oc = Opencage::new(
        std::env::var("OPENCAGE_API_KEY")
            .expect("Please set OpenCage API key as en environment variable OPENCAGE_API_KEY"),
    );
    oc.parameters.limit = Some("1");
    oc.parameters.language = Some(lang);
    oc
}

pub fn reverse_geocode(
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

pub fn reverse_geocode_nocache(
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

/// Returns [Place] with just fields, where both arguments share the same values
pub fn intersection(a: &Place, b: &Place) -> Place {
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
