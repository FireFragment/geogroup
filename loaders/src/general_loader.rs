use super::*;

/// A loader combining all the loaders in this crate
pub struct GeneralLoader;

/// Return value when trying to load data from a file.
pub type LoadingReturnValue = Result<
    LoaderSpecificLocData<GeneralLoader>,
    <general_loader::GeneralLoader as DataLoader>::FatalError,
>;

impl DataLoader for GeneralLoader {
    type FatalError = FatalError;
    type LocationError = LocationError;
    type TimeError = TimeError;

    fn get_data(&self, file: &Path) -> LoadingReturnValue {
        if !file.is_file() {
            return Err(FatalError::NotAFile);
        };

        let extension = file
            .extension()
            .ok_or(FatalError::NoExtension)?
            .to_str()
            .ok_or(FatalError::NonUnicodeFileName)?
            .to_string();

        if ExifLoader.supported_extensions().contains(&extension) {
            match ExifLoader.get_data(file) {
                Ok(v) => Ok(v.convert_errors()),
                Err(e) => Err(LoaderSpecificFatalError::from(e).into()),
            }
        } else {
            Err(FatalError::FileTypeNotSupported(extension))
        }
    }

    fn supported_extensions(&self) -> Vec<String> {
        ExifLoader.supported_extensions()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FatalError {
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

    #[error(transparent)]
    LoaderSpecific(#[from] LoaderSpecificFatalError),
}

#[derive(thiserror::Error, Debug)]
pub enum LoaderSpecificFatalError {
    #[error(transparent)]
    Exif(#[from] <ExifLoader as DataLoader>::FatalError),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum LocationError {
    #[error(transparent)]
    Exif(#[from] <ExifLoader as DataLoader>::LocationError),
}

#[derive(thiserror::Error, Debug)]
pub enum TimeError {
    #[error(transparent)]
    Exif(#[from] <ExifLoader as DataLoader>::TimeError),
}
