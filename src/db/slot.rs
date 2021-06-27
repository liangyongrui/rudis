mod expirations;

use std::{
    collections::HashMap,
    convert::TryInto,
    sync::{atomic::AtomicU64, Arc},
};

use bytes::Bytes;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use tokio::sync::broadcast;

use self::expirations::{Expiration, ExpirationEntry};
use super::{
    data_type::{DataType, SimpleType},
    result::Result,
};

/// Entry in the key-value store
#[derive(Debug)]
pub struct Entry {
    /// Uniquely identifies this entry.
    pub id: u64,

    /// Stored data
    pub data: DataType,

    /// Instant at which the entry expires and should be removed from the
    /// database.
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct Slot {
    /// The key-value data. We are not trying to do anything fancy so a
    /// `std::collections::HashMap` works fine.
    pub entries: Arc<DashMap<String, Entry>>,

    /// The pub/sub key-space. Redis uses a **separate** key space for key-value
    /// and pub/sub. `rcc` handles this by using a separate `HashMap`.
    pub pub_sub: HashMap<String, broadcast::Sender<Bytes>>,

    /// Tracks key TTLs.
    expirations: Expiration,

    /// Identifier to use for the next expiration. Each expiration is associated
    /// with a unique identifier. See above for why.
    next_id: AtomicU64,
}

impl Slot {
    pub fn new() -> Self {
        let entries = Arc::new(DashMap::new());
        let expirations = Expiration::new(Arc::clone(&entries));
        Self {
            entries,
            pub_sub: HashMap::new(),
            expirations,
            next_id: AtomicU64::new(0),
        }
    }

    // pub fn del(&mut self, key)
    pub fn get_simple(&self, key: &str) -> Result<Option<SimpleType>> {
        match self.entries.get(key) {
            Some(s) => match s.value() {
                Entry {
                    data: DataType::SimpleType(st),
                    ..
                } => Ok(Some(st.clone())),
                _ => Err("类型错误".to_string()),
            },
            None => Ok(None),
        }
    }

    pub fn get_or_insert_entry(
        &self,
        key: &str,
        f: fn() -> (DataType, Option<DateTime<Utc>>),
    ) -> dashmap::mapref::one::RefMut<'_, String, Entry> {
        if !self.entries.contains_key(key) {
            let (data, expires_at) = f();
            let e = Entry {
                id: self.next_id(),
                data,
                expires_at,
            };
            self.entries.insert(key.to_owned(), e);
        }
        self.entries.get_mut(key).unwrap()
    }

    /// Get and increment the next insertion ID.
    pub fn next_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    // fn next_expiration(&self) -> Option<DateTime<Utc>> {
    //     self.expirations
    //         .keys()
    //         .next()
    //         .map(|expiration| expiration.0)
    // }

    pub fn exists(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn get_expires_at(&self, key: &str) -> Option<DateTime<Utc>> {
        self.entries.get(key).and_then(|t| t.expires_at)
    }

    /// Purge all expired keys and return the `Instant` at which the **next**
    /// key will expire. The background task will sleep until this instant.
    // pub fn purge_expired_keys(&mut self) -> Option<DateTime<Utc>> {
    //     if self.shutdown {
    //         // The database is shutting down. All handles to the shared state
    //         // have dropped. The background task should exit.
    //         return None;
    //     }

    //     // This is needed to make the borrow checker happy. In short, `lock()`
    //     // returns a `MutexGuard` and not a `&mut State`. The borrow checker is
    //     // not able to see "through" the mutex guard and determine that it is
    //     // safe to access both `state.expirations` and `state.entries` mutably,
    //     // so we get a "real" mutable reference to `State` outside of the loop.
    //     // let state = &mut *state;

    //     // Find all keys scheduled to expire **before** now.
    //     let now = Utc::now();

    //     // 因为只需要处理头部元素，所有这里每次产生一个新的迭代器是安全的, 等first_entry stable 可以替换
    //     while let Some((&(when, id), key)) = self.expirations.iter().next() {
    //         if when > now {
    //             // Done purging, `when` is the instant at which the next key
    //             // expires. The worker task will wait until this instant.
    //             return Some(when);
    //         }
    //         debug!("purge_expired_keys: {}", key);
    //         // The key expired, remove it
    //         self.entries.remove(key);
    //         self.expirations.remove(&(when, id));
    //     }
    //     None
    // }

    /// return (old value exist, notify)
    pub fn set_expires_at(&self, key: String, expires_at: DateTime<Utc>) -> (bool, bool) {
        todo!()
        // let notify = self
        //     .next_expiration()
        //     .map(|expiration| expiration > expires_at)
        //     .unwrap_or(true);
        // if let Some(entry) = self.entries.get_mut(&key) {
        //     if let Some(old_when) = entry.expires_at {
        //         self.expirations.remove(&(old_when, entry.id));
        //         self.expirations.insert((expires_at, entry.id), key);
        //     }
        //     entry.expires_at = Some(expires_at);
        //     (true, notify)
        // } else {
        //     (false, notify)
        // }
    }

    /// 调用之前需要自己保证原始值的value 为 simpleType 或 不存在
    pub async fn update_simple(
        &self,
        key: String,
        value: SimpleType,
        expires_at: Option<DateTime<Utc>>,
    ) -> Option<SimpleType> {
        let id = self.next_id();
        if let Some(expires_at) = expires_at {
            let _ = self
                .expirations
                .sender
                .send(ExpirationEntry {
                    id,
                    key: key.clone(),
                    expires_at,
                })
                .await;
        }
        let prev = self.entries.insert(
            key,
            Entry {
                id,
                data: value.into(),
                expires_at,
            },
        );
        prev.map(|t| t.data.try_into().unwrap())
    }

    pub fn remove(&self, key: &str) -> Option<DataType> {
        self.entries.remove(key).map(|prev| prev.1.data)
    }

    pub(crate) async fn set(
        &self,
        key: String,
        value: SimpleType,
        nxxx: Option<bool>,
        mut expires_at: Option<DateTime<Utc>>,
        keepttl: bool,
    ) -> Result<Option<SimpleType>> {
        let old_value = self.get_simple(&key)?;
        let need_update = if let Some(nx) = nxxx {
            // old_value = state.get_data(&key).cloned();
            let c = old_value.is_some();
            (nx && !c) || (!nx && c)
        } else {
            true
        };
        if !need_update {
            return Ok(old_value);
        }
        if keepttl {
            expires_at = self.get_expires_at(&key);
        }
        Ok(self.update_simple(key, value, expires_at).await)
    }

    /// Returns a `Receiver` for the requested channel.
    ///
    /// The returned `Receiver` is used to receive values broadcast by `PUBLISH`
    /// commands.
    pub(crate) fn subscribe(&self, key: String) -> broadcast::Receiver<bytes::Bytes> {
        todo!()
        // use std::collections::hash_map::Entry;
        // // If there is no entry for the requested channel, then create a new
        // // broadcast channel and associate it with the key. If one already
        // // exists, return an associated receiver.
        // match self.pub_sub.entry(key) {
        //     Entry::Occupied(e) => e.get().subscribe(),
        //     Entry::Vacant(e) => {
        //         // No broadcast channel exists yet, so create one.
        //         //
        //         // The channel is created with a capacity of `1024` messages. A
        //         // message is stored in the channel until **all** subscribers
        //         // have seen it. This means that a slow subscriber could result
        //         // in messages being held indefinitely.
        //         //
        //         // When the channel's capacity fills up, publishing will result
        //         // in old messages being dropped. This prevents slow consumers
        //         // from blocking the entire system.
        //         let (tx, rx) = broadcast::channel(1024);
        //         e.insert(tx);
        //         rx
        //     }
        // }
    }

    /// Publish a message to the channel. Returns the number of subscribers
    /// listening on the channel.
    pub(crate) fn publish(&self, key: &str, value: bytes::Bytes) -> usize {
        self
            .pub_sub
            .get(key)
            // On a successful message send on the broadcast channel, the number
            // of subscribers is returned. An error indicates there are no
            // receivers, in which case, `0` should be returned.
            .map(|tx| tx.send(value).unwrap_or(0))
            // If there is no entry for the channel key, then there are no
            // subscribers. In this case, return `0`.
            .unwrap_or(0)
    }
}
