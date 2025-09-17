use super::{super::*, Final};

use std::cell::OnceCell;
use std::sync::{Mutex, mpsc};
use std::sync::OnceLock;

// TODO: Terminate on drop
/// LazyGroup which may be in a ["final" state](Final) or an "initializing" state.
///
/// In initializing state, it just returns [StructureErr]s until it's initialized into the final state.
#[derive(Debug)]
pub struct Dynamic {
    //template: lazy_group::Template,
    status: Arc<Mutex<Status>>,
    /// If inhabited, we already have obtained a final group
    ///
    /// Must be inhabited when `finalized` is [`None`]
    ///
    /// Don't read this directly, use [`Self::get_final`] to initialize this field in case
    /// the final value has been received.
    finalized: OnceCell<Finalized>,
    /// Must not be [`None`] when `finalized` is not inhabited
    final_value_recv: Option<mpsc::Receiver<Finalized>>,
    /// When message is sent through this, we attempt to terminate the initializing thread
    ///
    /// Warning: if dropped, we also attempt to terminate.
    terminator: mpsc::Sender<()>
}

impl From<lazy_group::Final> for Dynamic {
    fn from(fin_group: Final) -> Self {
        let (tx, _rx) = mpsc::channel();
        Self {
            status: Arc::new(Mutex::new(Status { progress: None, msg: None, time: std::time::Instant::now() })),
            finalized: OnceCell::from(Finalized::Success(fin_group)),
            final_value_recv: None,
            terminator: tx,
        }
    }
}

impl Dynamic {
    /*pub fn template(&self) -> &lazy_group::Template {
        &self.template
    }*/

    /// Attempts to init [`Self::finalized`] if there's a received value from the initializing thread
    fn try_init_finalized(&self) {
        let new_finalized = self.final_value_recv.as_ref()
            .expect("Bug: Both Dynamic::finalized.get() and Dynamic::final_value_recv are `None`")
            .try_recv().ok();

        if let Some(finalized) = new_finalized {
            self.finalized.set(finalized);
        }
    }

    pub fn get_final(&self) -> Option<&Finalized> {
        self.try_init_finalized();
        self.finalized.get()
    }
    pub fn get_final_mut(&mut self) -> Option<&mut Finalized> {
        self.try_init_finalized();
        self.finalized.get_mut()
    }

    pub fn read<'s, T>(&'s self, reader: impl for<'mutex> Fn(View<'mutex, 's>) -> T) -> T {
        if let Some(finalized) = &self.get_final() {
            reader(match finalized {
                Finalized::Success(final_group) => View::Finished(final_group),
                Finalized::Terminated => View::Terminated,
            })
        } else {
            reader(View::Initializing(&self.status.lock().expect("Poisoned")))
        }
    }
}

#[derive(Debug)]
pub enum Finalized {
    Success(lazy_group::Final),
    Terminated
}

/// A "view" into [lazy_group::Dynamic]. [lazy_group::Dynamic] is mainly read converted into this form through [lazy_group::Dynamic::read]
pub enum View<'mutex, 's> {
    Finished(&'s lazy_group::Final),
    Initializing(&'mutex Status),
    Terminated
}

/// Version of [`lazy_group::dynamic::View`] which allows for mutation in the finished case
pub enum ViewMut<'mutex, 's> {
    Finished(&'s mut lazy_group::Final),
    Initializing(&'mutex Status),
    Terminated
}

/// A status sent from the thread where [lazy_group::Dynamic] initializes the lazy group
#[derive(Debug)]
pub struct Status {
    pub progress: Option<u16>,
    pub msg: Option<String>,
    /// When we got the info above from the initializing thread .
    /// Could be also used to check if the thread became stale or deadlocked.
    pub time: std::time::Instant
}

impl Dynamic {
    /// Creates a new [lazy_group::Dynamic] and starts initialization on a separate thread.
    pub fn new(template: lazy_group::Template) -> Self {
        let (terminator_tx, terminator_rx) = mpsc::channel();
        let (final_value_tx, final_value_recv) = mpsc::channel();
        let status = Arc::new(Mutex::new(Status { progress: None, msg: None, time: std::time::Instant::now() }.into()));
        let finalized = OnceCell::new();

        {
            let template = template.clone();
            let status = status.clone();
            std::thread::spawn(move || {
                log::debug!("Started a new thread for initializing a lazy group");

                let ret_val = template.into_final(&mut ProgressCallback::new(&mut |progress, msg| {
                    let mut status_handle = status.lock().expect("Poisoned");
                    *status_handle = Status {
                        progress,
                        msg: msg.map(|m| m.to_string()),
                        time: std::time::Instant::now()
                    }.into();

                    match terminator_rx.try_recv() {
                        Ok(()) => true, // Message was received, so we should terminate
                        Err(mpsc::TryRecvError::Empty) => false, // Message was not received, so we should not terminate
                        Err(mpsc::TryRecvError::Disconnected) => true, // Transmitter was dropped, we terminate
                    }
                }));

                final_value_tx.send(match ret_val {
                    Some(group) => Finalized::Success(group),
                    None => Finalized::Terminated
                }).unwrap_or_else(|err| log::debug!("Failed to send final value: {}", err));

                log::debug!("Finished initializing a lazy group");
            });
        }
        Self {
            //template,
            status,
            finalized,
            final_value_recv: Some(final_value_recv),
            terminator: terminator_tx
        }
    }
}
