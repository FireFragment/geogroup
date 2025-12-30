use nom_exif::ExifTag;
use std::fmt;

use super::*;

#[derive(Default)]
pub struct NomExifLoader(nom_exif::MediaParser);

impl MutDataLoader for NomExifLoader {
    type LocationError = TagError;
    type TimeError = TagError;
    type FatalError = FatalError;
    fn get_data_mut(
        &mut self,
        file: &Path,
    ) -> Result<LoaderSpecificLocData<NomExifLoader>, Self::FatalError> {
        let media_source = nom_exif::MediaSource::file_path(file).map_err(FatalError::MediaSource)?;
        let mut exif_data: nom_exif::ExifIter = self.0.parse(media_source).map_err(FatalError::Parse)?;

        let gps_info = exif_data.parse_gps_info();

        let mut lon = None;
        let mut lat = None;
        let mut time = None;

        while let Some(parsed_tag) = exif_data.next() {
            match parsed_tag.tag() {
                Some(ExifTag::GPSLongitude) => {
                    lon.replace(get_urational(&parsed_tag).cloned());
                }
                Some(ExifTag::GPSLatitude) => {
                    lat.replace(get_urational(&parsed_tag).cloned());
                }
                Some(ExifTag::DateTimeOriginal) => {
                    time.replace(get_datetime(&parsed_tag).cloned());
                }
                _ => {
                    if lat.is_some() && lon.is_some() && time.is_some() {
                        break;
                    }
                }
            }
        }

        let time = time
            .ok_or(TagProblem::Missing)
            .flatten()
            .map_err(|problem| TagError {
                tag: ExifTag::DateTimeOriginal,
                problem,
            })
            .map(|t| [t, t]);

        let lon = lon
            .ok_or(TagProblem::Missing)
            .flatten()
            .map_err(|problem| TagError {
                tag: ExifTag::GPSLongitude,
                problem,
            });

        let lat = lat
            .ok_or(TagProblem::Missing)
            .flatten()
            .map_err(|problem| TagError {
                tag: ExifTag::GPSLatitude,
                problem,
            });

        let location = lon
            .map(|lon| {
                lat.map(|lat| {
                    let point = geo::Point::new(lon.as_float(), lat.as_float());
                    geo::Rect::new(point, point)
                })
            })
            .flatten();

        Ok(LoaderSpecificLocData::<NomExifLoader> { time, location })
    }

    fn supported_extensions_m(&self) -> Vec<String> {
        vec!["png".into(), "jpg".into()]
    }
}

/// Also logs warning in case of failure
fn get_urational<'a>(
    parsed_tag: &'a nom_exif::ParsedExifEntry,
) -> Result<&'a nom_exif::URational, TagProblem> {
    match parsed_tag.get_result() {
        Ok(nom_exif::EntryValue::URational(value)) => Ok(value),
        Ok(another_value) => Err(TagProblem::WrongExifType {
            bad_value: another_value.clone(),
            exp_type: String::from("URational"),
        }),
        Err(err) => Err(TagProblem::Failed(err.to_string())),
    }
}

/// Also logs warning in case of failure
fn get_datetime<'a>(
    parsed_tag: &'a nom_exif::ParsedExifEntry,
) -> Result<&'a chrono::DateTime<chrono::FixedOffset>, TagProblem> {
    match parsed_tag.get_result() {
        Ok(nom_exif::EntryValue::Time(value)) => Ok(value),
        Ok(another_value) => Err(TagProblem::WrongExifType {
            bad_value: another_value.clone(),
            exp_type: String::from("Time"),
        }),
        Err(err) => Err(TagProblem::Failed(err.to_string())),
    }
}

#[derive(Error, Debug, Clone)]
#[error("EXIF tag {tag} {}", problem.sentence_ending())]
pub struct TagError {
    tag: ExifTag,
    problem: TagProblem,
}

#[derive(Debug, Clone)]
pub enum TagProblem {
    Missing,
    WrongExifType {
        bad_value: nom_exif::EntryValue,
        exp_type: String,
    },
    /// Converted from nom_exif::EntryError
    Failed(String),
}

impl TagProblem {
    fn sentence_ending(&self) -> impl fmt::Display + '_ {
        struct Display<'a>(&'a TagProblem);
        impl fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match &self.0 {
                    TagProblem::Missing => write!(f, "is missing"),
                    TagProblem::WrongExifType { bad_value, exp_type } => write!(f, "has unexpected type:\n  Expected type: {exp_type}\n  Found value: {bad_value:?}"),
                    TagProblem::Failed(error) => write!(f, "has not been parsed successfully: {error}"),
                }
            }
        }

        Display(self)
    }
}

#[derive(Error, Debug)]
pub enum FatalError {
    #[error("Error with EXIF reader: {0}")]
    MediaSource(nom_exif::Error),
    #[error("Error with EXIF reader: {0}")]
    Parse(nom_exif::Error),
}
