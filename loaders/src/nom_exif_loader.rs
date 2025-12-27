use std::{
    convert::Infallible,
    ops::ControlFlow,
    str::{from_utf8, Utf8Error},
};

use log::debug;
use nom_exif::ExifTag;

use super::*;

pub struct NomExifLoader(nom_exif::MediaParser);

impl DataLoader for NomExifLoader {
    type LocationError = Infallible;
    type TimeError = Infallible;
    type FatalError = FatalError;
    fn get_data(
        &self,
        file: &Path,
    ) -> Result<LoaderSpecificLocData<NomExifLoader>, Self::FatalError> {
        let media_source = nom_exif::MediaSource::file_path(file)?;

        let mut exif_data: nom_exif::ExifIter = self.0.parse(media_source)?;
        let mut lon = None;
        let mut lat = None;
        let mut last_err = None;

        while let Some(parsed_tag) = exif_data.next() {
            match parsed_tag.tag() {
                Some(tag @ ExifTag::GPSLongitude) => {
                    debug_assert!(lon.is_none());
                    lon = get_urational(file, &parsed_tag, tag);
                },
                Some(tag @ ExifTag::GPSLatitude) => {
                    debug_assert!(lon.is_none());
                    lat = get_urational(file, &parsed_tag, tag);
                },
                _ => {}
            }
        }

        if let Some(err) = last_err {}

        todo!()
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["png".into(), "jpg".into()]
    }
}

/// Also logs warning in case of failure
fn get_urational<'a>(
    file: &Path,
    parsed_tag: &'a nom_exif::ParsedExifEntry,
    tag: ExifTag,
) -> Option<&'a nom_exif::URational> {
    match parsed_tag.get_result() {
        Ok(nom_exif::EntryValue::URational(value)) => Some(value),
        Ok(another_value) => {
            log::warn!(
                file=*file.to_string_lossy(),
                err = "UnexpectedEXIFType",
                exif_failed_parsing_tag:? = tag,
                expected_exif_type = "URational",
                found_exif_value:? = another_value;
                "EXIF tag {tag} of file {file} was expected to be `URational`, but it's {another_value} instead",
                file=file.to_string_lossy()
            );
            None
        }
        Err(err) => {
            log::warn!(
                file=*file.to_string_lossy(),
                err:?,
                exif_failed_parsing_tag:? = tag;
                "Failed to parse EXIF tag {tag} of file {file}\nCause: {err}",
                file=file.to_string_lossy()
            );
            None
        }
    }
}


/// Also logs warning in case of failure
fn get_str<'a>(
    file: &Path,
    parsed_tag: &'a nom_exif::ParsedExifEntry,
    tag: ExifTag,
) -> Option<&'a nom_exif::URational> {
    match parsed_tag.get_result() {
        Ok(nom_exif::EntryValue::(value)) => Some(value),
        Ok(another_value) => {
            log::warn!(
                file=*file.to_string_lossy(),
                err = "UnexpectedEXIFType",
                exif_failed_parsing_tag:? = tag,
                expected_exif_type = "URational",
                found_exif_value:? = another_value;
                "EXIF tag {tag} of file {file} was expected to be `URational`, but it's {another_value} instead",
                file=file.to_string_lossy()
            );
            None
        }
        Err(err) => {
            log::warn!(
                file=*file.to_string_lossy(),
                err:?,
                exif_failed_parsing_tag:? = tag;
                "Failed to parse EXIF tag {tag} of file {file}\nCause: {err}",
                file=file.to_string_lossy()
            );
            None
        }
    }
}

#[derive(Error, Debug)]
pub enum FatalError {
    #[error(transparent)]
    ExifError(#[from] nom_exif::Error),
}
