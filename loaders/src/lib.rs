use chrono::DateTime;
use std::error::Error as StdError;
use std::path::Path;
use thiserror::Error;

#[cfg(feature = "exif")]
pub mod exif_loader;
pub use exif_loader::ExifLoader;

mod general_loader;
pub use general_loader::GeneralLoader;

#[derive(Clone, Debug)]
pub struct LocData<
    TimeError: StdError = <GeneralLoader as DataLoader>::TimeError,
    LocationError: StdError = <GeneralLoader as DataLoader>::LocationError,
> {
    /// Time range of the file.
    /// First element must always be before or equal to the second element.
    ///
    /// If the file corresponds to a single instant, both elements are equal.
    pub time: Result<[DateTime<chrono::FixedOffset>; 2], TimeError>,
    /// Rectangle containing the real location of the file, if in doubt
    ///
    /// It might be that the file corresponds to multiple locations (eg. a GPX file).
    /// In that case, this rect should contain all of them
    pub location: Result<geo::Rect, LocationError>,
}

impl<FromTimeError: StdError, FromLocationError: StdError>
    LocData<FromTimeError, FromLocationError>
{
    pub fn convert_errors<
        ToTimeError: StdError + From<FromTimeError>,
        ToLocationError: StdError + From<FromLocationError>,
    >(
        self,
    ) -> LocData<ToTimeError, ToLocationError> {
        LocData {
            time: self.time.map_err(From::from),
            location: self.location.map_err(From::from),
        }
    }
}

/// [`LocData`] with errors corresponding to errors of a specific [`DataLoader`]
pub type LoaderSpecificLocData<Loader> =
    LocData<<Loader as DataLoader>::TimeError, <Loader as DataLoader>::LocationError>;

#[derive(Error, Debug)]
pub enum LocationError {
    #[cfg(feature = "exif")]
    #[error(transparent)]
    ExifError(#[from] exif_loader::CommonError),
}

pub trait DataLoader {
    /// Error preventing loading time from a file
    type TimeError: StdError;
    /// Error preventing loading location from a file
    type LocationError: StdError;
    /// Error preventing loading *any* data from a file, other than already provided in [`GenericFatalError`]
    type FatalError: StdError;

    fn get_data(&self, file: &Path) -> Result<LoaderSpecificLocData<Self>, Self::FatalError>;

    /// List of file extensions this loader supports
    fn supported_extensions(&self) -> Vec<String>;
}
