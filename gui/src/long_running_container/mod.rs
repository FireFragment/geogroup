//! See [`LongRunningContainer`]
use super::*;
pub mod sorter;

/// Container which contains a value on which some long-running operations can be performed.
/// During these operations, the inner value becomes unavailable
pub trait LongRunningContainer {
    type Inner;
    type WorkingInfo;
    type CancellingInfo;
    type Error: std::error::Error;

    fn status(&self) -> Status<&Self::Inner, Self::WorkingInfo, Self::CancellingInfo, Self::Error>;
    fn status_mut(
        &mut self,
    ) -> Status<&mut Self::Inner, Self::WorkingInfo, Self::CancellingInfo, Self::Error>;

    /// Can be called only when [`status`](LongRunningContainer::status) returned an error where
    /// [`can_retry`](Error::can_retry) is [true].
    ///
    /// Returns [`Err`](Result::Err) if and only if we are not in retry-able state - that is,
    /// [`status`](LongRunningContainer::status) wouldn't return error with [`can_retry`](Error::can_retry)
    /// field set to [true].
    /// Other errors during retrying are reported through the
    /// [`status`](LongRunningContainer::status) method.
    fn retry(&mut self) -> Result<(), ()>;

    /// Return [`true`] if calling [`cancel`](LongRunningContainer::cancel) is possible right now. It may change during the
    fn can_cancel() -> bool;

    /// May try to cancel the operation, but it may also be ignored if cancellation is not feasible right now.
    ///
    ///  - If cancellation wasn't ignored, [`status`](LongRunningContainer::status) should **immediately** start returning
    ///    either [`Cancelling`](Status::Cancelling) or [`Cancelled`](Status::Cancelled).
    ///  - If cancellation was ignored, [`status`](LongRunningContainer::status) should continue returning
    ///    whatever it was returning before and continue its operations normally.
    fn cancel(&mut self);
}

pub enum Status<T, WorkingInfo, CancellingInfo, E> {
    Ready(T),
    Error {
        error: E,
        can_retry: bool,
    },
    Cancelled,
    Working {
        info: WorkingInfo,
        /// [`u8::MAX`] indicates 100%.
        progress: Option<u8>,
    },
    Cancelling(CancellingInfo),
}

//pub struct SorterContainer(Option<backend::algorithm::Sorter>);
