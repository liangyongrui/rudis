use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::data_type::DataType;
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dict {
    next_id: u64,
    pub inner: HashMap<Vec<u8>, Value>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Value {
    pub id: u64,
    pub data: DataType,
    pub expire_at: Option<DateTime<Utc>>,
}

impl Dict {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    pub fn last_write_op_id(&self) -> u64 {
        self.next_id
    }

    #[inline]
    pub fn d_exists(&self, key: &[u8]) -> bool {
        self.d_get(key).is_some()
    }

    #[inline]
    pub fn d_get(&self, key: &[u8]) -> Option<&Value> {
        self.get(key)
            .filter(|v| v.expire_at.filter(|x| *x <= Utc::now()).is_none())
    }

    #[inline]
    pub fn d_get_mut(&mut self, key: &[u8]) -> Option<&mut Value> {
        self.get_mut(key)
            .filter(|v| v.expire_at.filter(|x| *x <= Utc::now()).is_none())
    }

    /// todo 这里可能可以优化一下
    pub fn d_get_mut_or_insert_with<F: FnOnce() -> Value>(
        &mut self,
        key: Vec<u8>,
        f: F,
    ) -> &mut Value {
        match self.entry(key.clone()) {
            std::collections::hash_map::Entry::Occupied(mut o) => {
                if o.get().expire_at.filter(|x| *x <= Utc::now()).is_some() {
                    o.insert(f());
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(f());
            }
        }
        self.get_mut(&key).unwrap()
    }
}

impl Deref for Dict {
    type Target = HashMap<Vec<u8>, Value>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Dict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
