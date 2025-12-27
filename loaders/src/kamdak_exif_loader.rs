use std::str::{from_utf8, Utf8Error};

use super::*;

#[derive(Clone, Debug)]
pub struct KamdakExifLoader;

impl DataLoader for KamdakExifLoader {
    type LocationError = CommonError;
    type TimeError = TimeError;
    type FatalError = FatalError;
    fn get_data(&self, file: &Path) -> Result<LoaderSpecificLocData<KamdakExifLoader>, Self::FatalError> {
        let file = std::fs::File::open(file).map_err(FatalError::CannotReadFile)?;
        let exif_data =
            exif::Reader::new().read_from_container(&mut std::io::BufReader::new(&file))?;

        Ok(LocData {
            location: get_location(&exif_data),
            time: get_time(&exif_data),
        })
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["png".into(), "jpg".into()]
    }
}

fn get_time(exif_data: &exif::Exif) -> Result<[DateTime<chrono::FixedOffset>; 2], TimeError> {
    let date_str = format!(
        "{} {}",
        ascii_tag_as_str(exif_data, exif::Tag::DateTimeOriginal)?,
        ascii_tag_as_str(exif_data, exif::Tag::OffsetTimeOriginal)?
    );
    let date = DateTime::parse_from_str(&date_str, "%Y:%m:%d %H:%M:%S %:z")
        .map_err(|err| TimeError::InvalidDate { date_str, err })?;

    Ok([date, date])
}

fn get_location(exif_data: &exif::Exif) -> Result<geo::Rect, CommonError> {
    let coord = geo::Coord {
        x: rational_tag_as_f64(exif_data, exif::Tag::GPSLongitude)?,
        y: rational_tag_as_f64(exif_data, exif::Tag::GPSLatitude)?,
    };

    Ok(geo::Rect::new(coord, coord))
}

pub fn ascii_tag_as_str(exif_data: &exif::Exif, tag: exif::Tag) -> Result<&str, TimeError> {
    let field_val = &exif_data
        .get_field(tag, exif::In::PRIMARY)
        .ok_or(CommonError::TagMissing(tag))?
        .value;

    match field_val {
        exif::Value::Ascii(txt) => from_utf8(&txt[0][..]).map_err(|err| TimeError::NonAsciiData {
            err,
            tag,
            value: field_val.to_owned(),
        }),
        val => Err(CommonError::InvalidType {
            tag,
            value: val.to_owned(),
            allowed_types: vec!["Ascii".into()],
        }
        .into()),
    }
}

pub fn rational_tag_as_f64(exif_data: &exif::Exif, tag: exif::Tag) -> Result<f64, CommonError> {
    let field_val = &exif_data
        .get_field(tag, exif::In::PRIMARY)
        .ok_or(CommonError::TagMissing(tag))?
        .value;

    Ok(match field_val {
        exif::Value::Rational(v) => {
            let coords_orig = v.iter().map(|v| v.to_f64()).collect::<Vec<_>>();
            coords_orig[0] + coords_orig[1] / 60.0 + coords_orig[2] / 3600.0
        }
        exif::Value::SRational(v) => {
            let coords_orig = v.iter().map(|v| v.to_f64()).collect::<Vec<_>>();
            coords_orig[0] + coords_orig[1] / 60.0 + coords_orig[2] / 3600.0
        }
        val => Err(CommonError::InvalidType {
            tag,
            value: val.to_owned(),
            allowed_types: vec!["Rational".into(), "SRational".into()],
        })?,
    })
}
#[derive(Error, Debug)]
pub enum FatalError {
    #[error("cannot read file: {0}")]
    CannotReadFile(#[source] std::io::Error),

    #[error(transparent)]
    ExifError(#[from] exif::Error),
}

/// Error type including common errors for both for location fetching and datetime fetching
#[derive(Error, Debug, Clone)]
pub enum CommonError {
    #[error("location/date is not saved in the image - tag {0} missing")]
    TagMissing(exif::Tag),

    #[error("tag {tag} has invalid value {value:?} - only following types are allowed: {allowed_types:?}")]
    InvalidType {
        tag: exif::Tag,
        value: exif::Value,
        allowed_types: Vec<String>,
    },
}

#[derive(Error, Debug, Clone)]
pub enum TimeError {
    #[error("failed to parse date string `{date_str}`. Expected format: `%Y:%m:%d %H:%M:%S %:z`")]
    InvalidDate {
        date_str: String,
        #[source]
        err: chrono::ParseError,
    },

    #[error("tag {tag} has invalid value {value:?} - it contains non-UTF8 characters")]
    NonAsciiData {
        tag: exif::Tag,
        value: exif::Value,
        #[source]
        err: Utf8Error,
    },

    #[error(transparent)]
    Other(#[from] CommonError),
}
