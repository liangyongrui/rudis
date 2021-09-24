#![allow(unstable_name_collisions)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::must_use_candidate)]

pub mod cmd;
pub mod data_type;

use std::collections::{BTreeSet, HashMap};

use common::now_timestamp_ms;
use data_type::DataType;
use keys::Key;
use serde::{Deserialize, Serialize};

pub trait Dict {
    fn next_id(&mut self) -> u64;

    fn last_write_op_id(&self) -> u64;

    fn set_write_id(&mut self, id: u64);

    fn exists(&mut self, key: &[u8]) -> bool;

    fn get(&mut self, key: &[u8]) -> Option<&mut Value>;

    fn get_or_insert_with<F: FnOnce() -> Value>(&mut self, key: Key, f: F) -> &mut Value;

    fn remove(&mut self, key: &[u8]) -> Option<Value>;

    fn insert(&mut self, k: Key, v: Value) -> Option<Value>;

    fn raw_get(&self, k: &[u8]) -> Option<&Value>;

    fn raw_get_mut(&mut self, k: &[u8]) -> Option<&mut Value>;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MemDict {
    pub write_id: u64,
    pub inner: HashMap<Key, Value, ahash::RandomState>,
    // todo
    pub lru_pool: BTreeSet<(u64, Key)>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Value {
    pub data: DataType,
    /// unix timestamp ms
    /// 0 表示不过期
    pub expires_at: u64,
    pub last_visit_time: u64,
}

impl Dict for MemDict {
    #[inline]
    fn next_id(&mut self) -> u64 {
        self.write_id += 1;
        self.write_id
    }

    fn last_write_op_id(&self) -> u64 {
        self.write_id
    }

    fn set_write_id(&mut self, id: u64) {
        self.write_id = id;
    }

    #[inline]
    fn exists(&mut self, key: &[u8]) -> bool {
        self.get(key).is_some()
    }

    #[inline]
    #[inline]
    fn get(&mut self, key: &[u8]) -> Option<&mut Value> {
        self.inner
            .get_mut(key)
            .filter(|v| v.expires_at == 0 || v.expires_at > now_timestamp_ms())
            .map(|v| {
                let now = now_timestamp_ms();
                v.last_visit_time = now;
                v
            })
    }

    #[inline]
    fn get_or_insert_with<F: FnOnce() -> Value>(&mut self, key: Key, f: F) -> &mut Value {
        match self.inner.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut o) => {
                let expires_at = o.get().expires_at;
                let now = now_timestamp_ms();
                if expires_at > 0 && expires_at <= now {
                    *o.get_mut() = f();
                } else {
                    o.get_mut().last_visit_time = now;
                }
                o.into_mut()
            }
            std::collections::hash_map::Entry::Vacant(e) => e.insert(f()),
        }
    }

    #[inline]
    fn remove(&mut self, key: &[u8]) -> Option<Value> {
        self.inner.remove(key)
    }

    #[inline]
    fn insert(&mut self, k: Key, v: Value) -> Option<Value> {
        self.inner.insert(k, v)
    }

    #[inline]
    fn raw_get(&self, k: &[u8]) -> Option<&Value> {
        self.inner.get(k)
    }

    fn raw_get_mut(&mut self, k: &[u8]) -> Option<&mut Value> {
        self.inner.get_mut(k)
    }

    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}
