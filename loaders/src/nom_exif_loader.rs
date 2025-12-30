use nom_exif::ExifTag;
use std::fmt;

use super::*;

#[derive(Default)]
pub struct NomExifLoader(nom_exif::MediaParser);

impl MutDataLoader for NomExifLoader {
    type LocationError = LocationError;
    type TimeError = TagError;
    type FatalError = FatalError;
    fn get_data_mut(
        &mut self,
        file: &Path,
    ) -> Result<LoaderSpecificLocData<NomExifLoader>, Self::FatalError> {
        let media_source =
            nom_exif::MediaSource::file_path(file).map_err(FatalError::MediaSource)?;
        let mut exif_data: nom_exif::ExifIter =
            self.0.parse(media_source).map_err(FatalError::Parse)?;

        let location = exif_data
            .parse_gps_info()
            .map_err(|e| LocationError::Error(e.to_string()))
            .map(|r| r.ok_or(LocationError::LocationMissing))
            .flatten()
            .map(
                |nom_exif::GPSInfo {
                     latitude,
                     longitude,
                     ..
                 }| {
                    geo::Point::new(
                        longitude.0.as_float()
                            + longitude.1.as_float() / 60.0
                            + longitude.2.as_float() / 3600.0,
                        latitude.0.as_float()
                            + latitude.1.as_float() / 60.0
                            + latitude.2.as_float() / 3600.0,
                    )
                },
            )
            .map(|p| geo::Rect::new(p, p));
        let mut time = None;

        while let Some(parsed_tag) = exif_data.next() {
            match parsed_tag.tag() {
                Some(ExifTag::DateTimeOriginal) => {
                    time.replace(get_datetime(&parsed_tag).cloned());
                }
                _ => {
                    if time.is_some() {
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

        Ok(LoaderSpecificLocData::<NomExifLoader> { time, location })
    }

    fn supported_extensions_m(&self) -> Vec<String> {
        vec!["png".into(), "jpg".into()]
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

#[derive(Clone, Error, Debug)]
pub enum LocationError {
    #[error("location is not saved in the file")]
    LocationMissing,

    /// Converted from [`nom_exif::Error`] to make this struct [`Clone`]
    #[error("file contains location, but parsing failed: {0}")]
    Error(String),
}

#[derive(Error, Debug)]
pub enum FatalError {
    #[error("error with EXIF reader: {0}")]
    MediaSource(nom_exif::Error),
    #[error("error with EXIF reader: {0}")]
    Parse(nom_exif::Error),
}
