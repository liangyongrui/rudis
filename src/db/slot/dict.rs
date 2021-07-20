use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use super::{expirations::ExpirationEntry, DataType};
use crate::SimpleType;

/// Entry in the key-value store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Uniquely identifies this entry.
    pub id: u64,

    /// Stored data
    pub data: DataType,

    /// Instant at which the entry expires and should be removed from the
    /// database.
    pub expires_at: Option<DateTime<Utc>>,
}

/// 每个slot 内部存储的数据
#[derive(Clone)]
pub struct Dict {
    /// 这里只有最底层的操作，直接上同步锁，不会锁太久
    inner: Arc<RwLock<DictInner>>,
}

impl Dict {
    pub fn read_lock(
        &self,
    ) -> parking_lot::lock_api::RwLockReadGuard<parking_lot::RawRwLock, DictInner> {
        self.inner.read()
    }

    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(DictInner::new())),
        }
    }

    pub fn get_expires_at(&self, key: &SimpleType) -> Option<DateTime<Utc>> {
        self.inner
            .read()
            .entries
            .get(key)
            .and_then(|t| t.expires_at)
            // 过期当key不存在
            .filter(|x| x > &Utc::now())
    }

    pub fn process_all<F: FnOnce(&DictInner) -> T, T>(&self, f: F) -> T {
        let mutex_guard = self.inner.read();
        f(mutex_guard.deref())
    }
    pub fn process_all_mut<F: FnOnce(&mut DictInner) -> T, T>(&self, f: F) -> T {
        let mut mutex_guard = self.inner.write();
        f(mutex_guard.deref_mut())
    }
    pub fn process_mut<F: FnOnce(Option<&mut Entry>) -> T, T>(&self, key: &SimpleType, f: F) -> T {
        let mut mutex_guard = self.inner.write();
        let res = mutex_guard
            .entries
            .get_mut(key)
            .filter(|x| match x.expires_at {
                Some(y) => y > Utc::now(),
                None => true,
            });
        f(res)
    }

    pub fn process<F: FnOnce(Option<&Entry>) -> T, T>(&self, key: &SimpleType, f: F) -> T {
        let mutex_guard = self.inner.read();
        let res = mutex_guard.entries.get(key).filter(|x| match x.expires_at {
            Some(y) => y > Utc::now(),
            None => true,
        });
        f(res)
    }

    /// new id
    pub fn update_expires_at<F: FnOnce() -> u64>(
        &self,
        key: &SimpleType,
        expires_at: Option<DateTime<Utc>>,
        next_id: F,
    ) -> Option<u64> {
        let mut gurad = self.inner.write();
        if let Some(v) = gurad.entries.get_mut(&key) {
            let id = next_id();
            *v = Entry {
                id,
                data: v.data.clone(),
                expires_at,
            };
            Some(id)
        } else {
            None
        }
    }

    pub fn get(&self, key: &SimpleType) -> Option<Entry> {
        self.inner
            .read()
            .entries
            .get(key)
            .filter(|x| match x.expires_at {
                Some(y) => y > Utc::now(),
                None => true,
            })
            .cloned()
    }

    pub fn exists(&self, key: &SimpleType) -> bool {
        self.inner
            .read()
            .entries
            .get(key)
            .filter(|x| match x.expires_at {
                Some(y) => y > Utc::now(),
                None => true,
            })
            .is_some()
    }

    /// return old_value
    pub fn insert_or_update(&self, key: SimpleType, entry: Entry) -> Option<Entry> {
        let mut mutex_guard = self.inner.write();
        mutex_guard.entries.insert(key, entry)
    }

    pub fn remove(&self, key: &SimpleType) -> Option<Entry> {
        self.inner.write().entries.remove(key)
    }

    /// return (result, 是否新插入)
    pub fn get_or_insert<F: FnOnce(&mut Entry) -> T, T, F2: FnOnce() -> Entry>(
        &self,
        key: SimpleType,
        f: F2,
        then_do: F,
    ) -> (T, Option<ExpirationEntry>) {
        let mut gurad = self.inner.write();
        match gurad.entries.entry(key.clone()) {
            std::collections::hash_map::Entry::Occupied(mut e) => (then_do(e.get_mut()), None),
            std::collections::hash_map::Entry::Vacant(e) => {
                let value = f();
                let res = value.expires_at.map(|expires_at| ExpirationEntry {
                    id: value.id,
                    key,
                    expires_at,
                });
                (then_do(e.insert(value)), res)
            }
        }
    }
}

pub struct DictInner {
    pub entries: HashMap<SimpleType, Entry>,
}

impl DictInner {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}
