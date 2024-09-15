use chrono::DateTime;
use std::error::Error as StdError;
use std::path::PathBuf;
use thiserror::Error;

#[cfg(feature = "exif")]
pub mod exif_loader;

#[derive(Clone, Debug)]
pub struct LocData<Loader: DataLoader + ?Sized> {
    /// Time range of the file.
    /// First element must always be before or equal to the second element.
    ///
    /// If the file corresponds to a single instant, both elements are equal.
    pub time: Result<[DateTime<chrono::FixedOffset>; 2], Loader::TimeError>,
    /// Rectangle containing the real location of the file, if in doubt
    ///
    /// It might be that the file corresponds to multiple locations (eg. a GPX file).
    /// In that case, this rect should contain all of them
    pub location: Result<geo::Rect, Loader::LocationError>,
}

#[derive(Error, Debug)]
pub enum LocationError {
    #[cfg(feature = "exif")]
    #[error(transparent)]
    ExifError(#[from] exif_loader::CommonError),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Given file type is not supported
    #[error("files with extension `.{0}` are not supported")]
    FileTypeNotSupported(
        /// Extension of the problematic file without the dot (eg. `mp3`)
        String,
    ),

    /// File name has non-unicode characters in it
    #[error("file name has non-unicode characters in it")]
    NonUnicodeFileName,

    /// File has no extension
    #[error("file has no extension")]
    NoExtension,

    /// Got something different than file
    #[error("expected file")]
    NotAFile,
}

/*pub fn get_data(file: PathBuf) -> Result<LocData, Error> {
    if !file.is_file() {
        return Err(Error::NotAFile);
    };

    let extension = file
        .extension()
        .ok_or(Error::NoExtension)?
        .to_str()
        .ok_or(Error::NonUnicodeFileName)?;

    match extension {
        "jpg" | "png" => todo!(),
        ext => Err(Error::FileTypeNotSupported(ext.into())),
    }
}*/

pub trait DataLoader {
    /// Error preventing loading time from a file
    type TimeError: StdError;
    /// Error preventing loading location from a file
    type LocationError: StdError;
    /// Error preventing loading *any* data from a file, other than already provided in [`GenericFatalError`]
    type FatalError: StdError;

    fn get_data(&self, file: PathBuf)
        -> Result<LocData<Self>, GenericFatalError<Self::FatalError>>;
}

/// A generic [`FatalError`](LocDataLoader::FatalError) that can be thrown by any [`DataLoader`]
#[derive(Error, Debug)]
pub enum GenericFatalError<E: StdError> {
    /// File cannot be read
    #[error("cannot read file")]
    CannotReadFile(std::io::Error),
    #[error(transparent)]
    Other(#[from] E),
}
