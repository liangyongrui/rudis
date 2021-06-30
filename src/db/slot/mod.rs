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
use crate::options::NxXx;

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

    pub fn exists(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn get_expires_at(&self, key: &str) -> Option<DateTime<Utc>> {
        self.entries.get(key).and_then(|t| t.expires_at)
    }

    pub fn set_expires_at(&self, key: String, new_time: DateTime<Utc>) -> bool {
        self.entries
            .get(&key)
            .and_then(|t| t.expires_at.map(|p| (p, t.id)))
            .map(|(pre_time, id)| self.expirations.update(id, pre_time, new_time))
            .unwrap_or(false)
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
        nx_xx: NxXx,
        mut expires_at: Option<DateTime<Utc>>,
        keepttl: bool,
    ) -> Result<Option<SimpleType>> {
        let old_value = self.get_simple(&key)?;
        let need_update = match nx_xx {
            NxXx::Nx => !old_value.is_some(),
            NxXx::Xx => old_value.is_some(),
            NxXx::None => true,
        };
        if !need_update {
            return Ok(old_value);
        }
        if keepttl {
            expires_at = self.get_expires_at(&key);
        }
        Ok(self.update_simple(key, value, expires_at).await)
    }
}
