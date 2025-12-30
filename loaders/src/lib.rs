use chrono::DateTime;
use std::error::Error as StdError;
use std::path::Path;
use thiserror::Error;

use geogroup_common as common;

#[cfg(feature = "exif")]
pub mod kamdak_exif_loader;
pub use kamdak_exif_loader as exif_loader;
pub use exif_loader::KamdakExifLoader as ExifLoader;


#[cfg(feature = "exif")]
pub mod nom_exif_loader;
pub use nom_exif_loader::NomExifLoader;

pub mod general_loader;
pub use general_loader::GeneralLoader;
pub use general_loader::LoadingReturnValue;

#[derive(Clone, Debug)]
pub struct LocData<
    TimeError = <GeneralLoader as MutDataLoader>::TimeError,
    LocationError = <GeneralLoader as MutDataLoader>::LocationError,
> {
    /// Time range of the file.
    /// First element must always be before or equal to the second element.
    ///
    /// If the file corresponds to a single instant, both elements are equal.
    time: Result<[DateTime<chrono::FixedOffset>; 2], TimeError>,
    /// Rectangle containing the real location of the file, if in doubt
    ///
    /// It might be that the file corresponds to multiple locations (eg. a GPX file).
    /// In that case, this rect should contain all of them
    location: Result<geo::Rect, LocationError>,
}

#[derive(Error, Debug)]
pub enum TimeOrLocError<T, L> {
    #[error("failed to get datetime of an item: {0}")]
    TimeError(T),

    #[error("failed to get location of an item: {0}")]
    LocationError(L),
}

impl<TimeError, LocationError> LocData<TimeError, LocationError> {
    pub fn as_sortable_item<D>(
        &self,
        data: D
    ) -> Result<
        common::ConcreteSortableItem<geo::Point, DateTime<chrono::FixedOffset>, D, &LocationError>,
        &TimeError,
    > {
        // For now, we ignore the "deltas" and just return everything as an average of values
        Ok(common::ConcreteSortableItem {
            position: self
                .location
                .as_ref()
                .map(|b| b.center().into()),
            time: {
                let [start, end] = self
                    .time
                    .as_ref()?;
                *start + (*end - *start) / 2
            },
            data,
        })
    }

    pub fn into_sortable_item<D>(
        self,
        data: D
    ) -> Result<
        common::ConcreteSortableItem<geo::Point, DateTime<chrono::FixedOffset>, D, LocationError>,
        TimeError,
    > where LocationError: Clone {
        // For now, we ignore the "deltas" and just return everything as an average of values
        Ok(common::ConcreteSortableItem {
            position: self
                .location
                .map(|b| b.center().into()),
            time: {
                let [start, end] = self
                    .time?;
                start + (end - start) / 2
            },
            data,
        })
    }
}

impl<FromTimeError, FromLocationError> LocData<FromTimeError, FromLocationError> {
    pub fn convert_errors<
        ToTimeError: From<FromTimeError>,
        ToLocationError: From<FromLocationError>,
    >(
        self,
    ) -> LocData<ToTimeError, ToLocationError> {
        LocData {
            time: self.time.map_err(From::from),
            location: self.location.map_err(From::from),
        }
    }

    pub fn map_errors<
        ToTimeError,
        ToLocationError,
        TimeFn: FnOnce(FromTimeError) -> ToTimeError,
        LocFn: FnOnce(FromLocationError) -> ToLocationError,
    >(
        self,
        time_fn: TimeFn,
        loc_fn: LocFn,
    ) -> LocData<ToTimeError, ToLocationError> {
        LocData {
            time: self.time.map_err(time_fn),
            location: self.location.map_err(loc_fn),
        }
    }
}

/// [`LocData`] with errors corresponding to errors of a specific [`DataLoader`]
pub type LoaderSpecificLocData<Loader> =
    LocData<<Loader as MutDataLoader>::TimeError, <Loader as MutDataLoader>::LocationError>;

#[derive(Error, Debug)]
pub enum LocationError {
    #[cfg(feature = "exif")]
    #[error(transparent)]
    ExifError(#[from] exif_loader::CommonError),
}

pub trait ImmutDataLoader {
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

pub trait MutDataLoader {
    /// Error preventing loading time from a file
    type TimeError: StdError;
    /// Error preventing loading location from a file
    type LocationError: StdError;
    /// Error preventing loading *any* data from a file, other than already provided in [`GenericFatalError`]
    type FatalError: StdError;

    fn get_data_mut(&mut self, file: &Path) -> Result<LoaderSpecificLocData<Self>, Self::FatalError>;

    /// List of file extensions this loader supports. Same as
    fn supported_extensions_m(&self) -> Vec<String>;
}

impl<T: ImmutDataLoader + ?Sized> MutDataLoader for T {
    type TimeError = <Self as ImmutDataLoader>::TimeError;
    type LocationError = <Self as ImmutDataLoader>::LocationError;
    type FatalError = <Self as ImmutDataLoader>::FatalError;

    fn get_data_mut(&mut self, file: &Path) -> Result<LoaderSpecificLocData<Self>, Self::FatalError> {
        ImmutDataLoader::get_data(&*self, file)
    }

    fn supported_extensions_m(&self) -> Vec<String> {
        ImmutDataLoader::supported_extensions(&*self)
    }
}

/*pub fn get_data(file: &Path) -> LoadingReturnValue {
    GeneralLoader::default().get_data_mut(file)
}
*/
