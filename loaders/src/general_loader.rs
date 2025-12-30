use super::*;

/// A loader combining all the loaders in this crate
#[derive(Default)]
pub struct GeneralLoader(ExifLoader);


/// Return value when trying to load data from a file.
pub type LoadingReturnValue = Result<
    LoaderSpecificLocData<GeneralLoader>,
    <general_loader::GeneralLoader as MutDataLoader>::FatalError,
>;

impl MutDataLoader for GeneralLoader {
    type FatalError = FatalError;
    type LocationError = LocationError;
    type TimeError = TimeError;

    fn get_data_mut(&mut self, file: &Path) -> LoadingReturnValue {
        if !file.is_file() {
            return Err(FatalError::NotAFile);
        };

        let extension = file
            .extension()
            .ok_or(FatalError::NoExtension)?
            .to_str()
            .ok_or(FatalError::NonUnicodeFileName)?
            .to_string();

        if self.0.supported_extensions_m().contains(&extension) {
            match self.0.get_data_mut(file) {
                Ok(v) => Ok(v.convert_errors()),
                Err(e) => Err(LoaderSpecificFatalError::from(e).into()),
            }
        } else {
            Err(FatalError::FileTypeNotSupported(extension))
        }
    }

    fn supported_extensions_m(&self) -> Vec<String> {
        self.0.supported_extensions_m()
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
    Exif(#[from] <ExifLoader as MutDataLoader>::FatalError),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum LocationError {
    #[error(transparent)]
    Exif(#[from] <ExifLoader as MutDataLoader>::LocationError),
}

#[derive(thiserror::Error, Debug)]
pub enum TimeError {
    #[error(transparent)]
    Exif(#[from] <ExifLoader as MutDataLoader>::TimeError),
}
