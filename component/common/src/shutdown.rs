use tokio::sync::broadcast::{self, error::TryRecvError};

/// Listens for the server shutdown signal.
///
/// Shutdown is signalled using a `broadcast::Receiver`. Only a single value is
/// ever sent. Once a value has been sent via the broadcast channel, the server
/// should shutdown.
///
/// The `Shutdown` struct listens for the signal and tracks that the signal has
/// been received. Callers may query for whether the shutdown signal has been
/// received or not.
#[derive(Debug)]
pub struct Shutdown {
    /// `true` if the shutdown signal has been received
    shutdown: bool,

    /// The receive half of the channel used to listen for shutdown.
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    /// Create a new `Shutdown` backed by the given `broadcast::Receiver`.
    #[inline]
    #[must_use]
    pub fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            shutdown: false,
            notify,
        }
    }

    /// Returns `true` if the shutdown signal has been received.
    #[inline]
    #[must_use]
    pub const fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    /// Returns `true` if the shutdown signal has been received.
    #[inline]
    pub fn check_shutdown(&mut self) -> bool {
        if Err(TryRecvError::Empty) == self.notify.try_recv() {
            false
        } else {
            self.shutdown = true;
            true
        }
    }

    /// Receive the shutdown notice, waiting if necessary.
    #[inline]
    pub async fn recv(&mut self) {
        // If the shutdown signal has already been received, then return
        // immediately.
        if self.shutdown {
            return;
        }

        // Cannot receive a "lag error" as only one value is ever sent.
        let _ = self.notify.recv().await;

        // Remember that the signal has been received.
        self.shutdown = true;
    }
}
