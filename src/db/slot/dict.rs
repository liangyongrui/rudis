use std::{
    collections::{hash_map, HashMap},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
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
    inner: Arc<Mutex<DictInner>>,
}

impl Dict {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DictInner::new())),
        }
    }
    pub fn get_expires_at(&self, key: &SimpleType) -> Option<DateTime<Utc>> {
        self.inner
            .lock()
            .unwrap()
            .entries
            .get(key)
            .and_then(|t| t.expires_at)
            // 过期当key不存在
            .filter(|x| x > &Utc::now())
    }

    pub fn process_mut<F: FnOnce(Option<&mut Entry>) -> T, T>(&self, key: &SimpleType, f: F) -> T {
        let mut mutex_guard = self.inner.lock().unwrap();
        let res = mutex_guard
            .entries
            .get_mut(key)
            .filter(|x| match x.expires_at {
                Some(y) => y > Utc::now(),
                None => true,
            });
        f(res)
    }

    pub fn entry<F: FnOnce(hash_map::Entry<SimpleType, Entry>) -> T, T>(
        &self,
        key: SimpleType,
        f: F,
    ) -> T {
        f(self.inner.lock().unwrap().entries.entry(key))
    }

    pub fn get(&self, key: &SimpleType) -> Option<Entry> {
        self.inner
            .lock()
            .unwrap()
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
            .lock()
            .unwrap()
            .entries
            .get(key)
            .filter(|x| match x.expires_at {
                Some(y) => y > Utc::now(),
                None => true,
            })
            .is_some()
    }

    pub fn insert(&self, key: SimpleType, value: Entry) -> Option<Entry> {
        self.inner.lock().unwrap().entries.insert(key, value)
    }

    pub fn remove(&self, key: &SimpleType) -> Option<Entry> {
        self.inner.lock().unwrap().entries.remove(key)
    }

    /// return (result, 需要更新的id和过期时间)
    pub fn get_or_insert<F: FnOnce(&mut Entry) -> T, T>(
        &self,
        key: SimpleType,
        f: fn() -> (DataType, Option<DateTime<Utc>>),
        then_do: F,
    ) -> (T, Option<ExpirationEntry>) {
        let mut gurad = self.inner.lock().unwrap();
        if let Some(v) = gurad.entries.get_mut(&key) {
            return (then_do(v), None);
        }
        let id = gurad.next_id();
        let (data, expires_at) = f();
        let mut v = Entry {
            id,
            data,
            expires_at,
        };
        let res = then_do(&mut v);
        gurad.entries.insert(key.clone(), v);
        drop(gurad);
        (
            res,
            expires_at.map(|expires_at| ExpirationEntry {
                id,
                key,
                expires_at,
            }),
        )
    }
}

pub struct DictInner {
    entries: HashMap<SimpleType, Entry>,
    next_id: u64,
}

impl DictInner {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_id: 0,
        }
    }

    fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}
