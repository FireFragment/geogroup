use super::*;
pub struct ProgressCallback<'a>(Box<dyn FnMut(Option<u16>, Option<&str>) -> bool + 'a>);

impl<'a> ProgressCallback<'a> {
    /// Returns [`true`] if the operation should be terminated
    #[must_use]
    fn call(&mut self, progress: Option<u16>, msg: Option<&str>) -> bool {
        (self.0)(progress, msg)
    }
}

/// # Constructors
impl<'a> ProgressCallback<'a> {
    pub fn new(fun: &'a mut impl FnMut(Option<u16>, Option<&str>) -> bool) -> Self {
        ProgressCallback(Box::new(fun))
    }

    pub fn new_ignore() -> Self {
        ProgressCallback(Box::new(|_, _| false))
    }
}

impl<T: Iterator> TerminatableIterator for T {}

pub trait TerminatableIterator: Iterator {
    /// `frequency` is how often is the callback called - 0 is every element, 1 is every second, 2 is every third, etc.
    fn with_progress_callback(
        self,
        mut callback: ProgressCallback,
        msg_generator: impl Fn(&Self::Item) -> Option<String>,
        interval: usize,
    ) -> impl Iterator<Item = Self::Item>
    where
        Self: Sized,
    {
        let (_, est_size) = self.size_hint();
        self.enumerate().map(move |(idx, it)| {
            if idx % (interval + 1) == 0 {
                let should_terminate = callback.call(
                    est_size.map(|est_size| ((idx as f32 / est_size as f32) * u16::MAX as f32) as u16),
                    msg_generator(&it).as_ref().map(|x| x.as_str()),
                );
                if should_terminate {
                    todo!()
                }
            }
            it
        })
    }
}
