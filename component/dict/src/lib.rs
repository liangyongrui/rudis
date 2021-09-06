#![allow(unstable_name_collisions)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::must_use_candidate)]

pub mod cmd;
pub mod data_type;

use std::collections::HashMap;

use common::now_timestamp_ms;
use data_type::DataType;
use keys::Key;
use serde::{Deserialize, Serialize};
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dict {
    pub write_id: u64,
    pub inner: HashMap<Key, Value, ahash::RandomState>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Value {
    pub data: DataType,
    /// unix timestamp ms
    /// 0 表示不过期
    pub expires_at: u64,
}

impl Dict {
    #[inline]
    pub fn next_id(&mut self) -> u64 {
        self.write_id += 1;
        self.write_id
    }

    pub fn last_write_op_id(&self) -> u64 {
        self.write_id
    }

    #[inline]
    pub fn exists(&self, key: &[u8]) -> bool {
        self.get(key).is_some()
    }

    #[inline]
    pub fn get(&self, key: &[u8]) -> Option<&Value> {
        self.inner
            .get(key)
            .filter(|v| v.expires_at == 0 || v.expires_at > now_timestamp_ms())
    }

    #[inline]
    pub fn get_mut(&mut self, key: &[u8]) -> Option<&mut Value> {
        self.inner
            .get_mut(key)
            .filter(|v| v.expires_at == 0 || v.expires_at > now_timestamp_ms())
    }

    #[inline]
    pub fn get_mut_or_insert_with<F: FnOnce() -> Value>(&mut self, key: Key, f: F) -> &mut Value {
        match self.inner.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut o) => {
                let expires_at = o.get().expires_at;
                if expires_at > 0 && expires_at <= now_timestamp_ms() {
                    *o.get_mut() = f();
                }
                o.into_mut()
            }
            std::collections::hash_map::Entry::Vacant(e) => e.insert(f()),
        }
    }

    #[inline]
    pub fn remove(&mut self, key: &[u8]) -> Option<Value> {
        self.inner.remove(key)
    }

    #[inline]
    pub fn insert(&mut self, k: Key, v: Value) -> Option<Value> {
        self.inner.insert(k, v)
    }

    #[inline]
    pub fn raw_get(&self, k: &[u8]) -> Option<&Value> {
        self.inner.get(k)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
