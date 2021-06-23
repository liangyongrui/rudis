use std::collections::{BTreeMap, HashMap};

use bytes::Bytes;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use tracing::debug;

/// Entry in the key-value store
#[derive(Debug)]
struct Entry {
    /// Uniquely identifies this entry.
    id: u64,

    /// Stored data
    data: Bytes,

    /// Instant at which the entry expires and should be removed from the
    /// database.
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct State {
    /// The key-value data. We are not trying to do anything fancy so a
    /// `std::collections::HashMap` works fine.
    entries: HashMap<String, Entry>,

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
    expirations: BTreeMap<(DateTime<Utc>, u64), String>,

    /// Identifier to use for the next expiration. Each expiration is associated
    /// with a unique identifier. See above for why.
    next_id: u64,

    /// True when the Slot instance is shutting down. This happens when all `Slot`
    /// values drop. Setting this to `true` signals to the background task to
    /// exit.
    shutdown: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            pub_sub: HashMap::new(),
            expirations: BTreeMap::new(),
            next_id: 0,
            shutdown: false,
        }
    }

    /// Get and increment the next insertion ID.
    pub fn next_id(&mut self) -> u64 {
        let res = self.next_id;
        self.next_id += 1;
        res
    }

    fn next_expiration(&self) -> Option<DateTime<Utc>> {
        self.expirations
            .keys()
            .next()
            .map(|expiration| expiration.0)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn get_expires_at(&self, key: &str) -> Option<DateTime<Utc>> {
        self.entries.get(key).and_then(|t| t.expires_at)
    }

    pub fn get_data(&self, key: &str) -> Option<&Bytes> {
        self.entries.get(key).map(|t| &t.data)
    }

    /// Purge all expired keys and return the `Instant` at which the **next**
    /// key will expire. The background task will sleep until this instant.
    pub fn purge_expired_keys(&mut self) -> Option<DateTime<Utc>> {
        if self.shutdown {
            // The database is shutting down. All handles to the shared state
            // have dropped. The background task should exit.
            return None;
        }

        // This is needed to make the borrow checker happy. In short, `lock()`
        // returns a `MutexGuard` and not a `&mut State`. The borrow checker is
        // not able to see "through" the mutex guard and determine that it is
        // safe to access both `state.expirations` and `state.entries` mutably,
        // so we get a "real" mutable reference to `State` outside of the loop.
        // let state = &mut *state;

        // Find all keys scheduled to expire **before** now.
        let now = Utc::now();

        // 因为只需要处理头部元素，所有这里每次产生一个新的迭代器是安全的, 等first_entry stable 可以替换
        while let Some((&(when, id), key)) = self.expirations.iter().next() {
            if when > now {
                // Done purging, `when` is the instant at which the next key
                // expires. The worker task will wait until this instant.
                return Some(when);
            }
            debug!("purge_expired_keys: {}", key);
            // The key expired, remove it
            self.entries.remove(key);
            self.expirations.remove(&(when, id));
        }
        None
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    pub fn shutdown(&mut self, s: bool) {
        self.shutdown = s;
    }

    /// return (old value exist, notify)
    pub fn set_expires_at(&mut self, key: String, expires_at: DateTime<Utc>) -> (bool, bool) {
        let notify = self
            .next_expiration()
            .map(|expiration| expiration > expires_at)
            .unwrap_or(true);
        if let Some(entry) = self.entries.get_mut(&key) {
            if let Some(old_when) = entry.expires_at {
                self.expirations.remove(&(old_when, entry.id));
                self.expirations.insert((expires_at, entry.id), key);
            }
            entry.expires_at = Some(expires_at);
            (true, notify)
        } else {
            (false, notify)
        }
    }

    /// get old data and update
    pub fn update(
        &mut self,
        key: String,
        value: Bytes,
        expires_at: Option<DateTime<Utc>>,
    ) -> (Option<Bytes>, bool) {
        let id = self.next_id();
        let notify = if let Some(expires_at) = expires_at {
            let res = self
                .next_expiration()
                .map(|expiration| expiration > expires_at)
                .unwrap_or(true);

            // Track the expiration.
            self.expirations.insert((expires_at, id), key.clone());
            res
        } else {
            false
        };
        let prev = self.entries.insert(
            key,
            Entry {
                id,
                data: value,
                expires_at,
            },
        );
        let old_value = prev.map(|prev| {
            if let Some(when) = prev.expires_at {
                // clear expiration
                self.expirations.remove(&(when, prev.id));
            }
            prev.data
        });
        (old_value, notify)
    }

    pub fn remove(&mut self, key: &str) -> Option<Bytes> {
        self.entries.remove(key).map(|prev| {
            if let Some(when) = prev.expires_at {
                // clear expiration
                self.expirations.remove(&(when, prev.id));
            }
            prev.data
        })
    }
}
