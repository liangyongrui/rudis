use std::collections::{BTreeMap, HashMap};

use bytes::Bytes;
use tokio::{sync::broadcast, time::Instant};

/// Entry in the key-value store
#[derive(Debug)]
pub struct Entry {
    /// Uniquely identifies this entry.
    pub id: u64,

    /// Stored data
    pub data: Bytes,

    /// Instant at which the entry expires and should be removed from the
    /// database.
    pub expires_at: Option<Instant>,
}

#[derive(Debug)]
pub struct State {
    /// The key-value data. We are not trying to do anything fancy so a
    /// `std::collections::HashMap` works fine.
    pub entries: HashMap<String, Entry>,

    /// The pub/sub key-space. Redis uses a **separate** key space for key-value
    /// and pub/sub. `rcc` handles this by using a separate `HashMap`.
    pub pub_sub: HashMap<String, broadcast::Sender<Bytes>>,

    /// Tracks key TTLs.
    ///
    /// A `BTreeMap` is used to maintain expirations sorted by when they expire.
    /// This allows the background task to iterate this map to find the value
    /// expiring next.
    ///
    /// While highly unlikely, it is possible for more than one expiration to be
    /// created for the same instant. Because of this, the `Instant` is
    /// insufficient for the key. A unique expiration identifier (`u64`) is used
    /// to break these ties.
    pub expirations: BTreeMap<(Instant, u64), String>,

    /// Identifier to use for the next expiration. Each expiration is associated
    /// with a unique identifier. See above for why.
    pub next_id: u64,

    /// True when the Slot instance is shutting down. This happens when all `Slot`
    /// values drop. Setting this to `true` signals to the background task to
    /// exit.
    pub shutdown: bool,
}

impl State {
    pub fn next_expiration(&self) -> Option<Instant> {
        self.expirations
            .keys()
            .next()
            .map(|expiration| expiration.0)
    }
}
