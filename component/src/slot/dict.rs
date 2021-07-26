use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use chrono::{DateTime, Utc};

use super::data_type::{DataType, KeyType};
#[derive(Debug, Default)]
pub struct Dict {
    /// 最后一次写操作的id
    pub last_write_op_id: u64,
    pub inner: HashMap<KeyType, Value>,
}

#[derive(Debug, PartialEq, Eq)]
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
    pub fn d_exists(&self, key: &KeyType) -> bool {
        self.d_get(key).is_some()
    }

    #[inline]
    pub fn d_get(&self, key: &KeyType) -> Option<&Value> {
        self.get(key)
            .filter(|v| v.expire_at.filter(|x| *x <= Utc::now()).is_none())
    }

    #[inline]
    pub fn d_get_mut(&mut self, key: &KeyType) -> Option<&mut Value> {
        self.get_mut(key)
            .filter(|v| v.expire_at.filter(|x| *x <= Utc::now()).is_none())
    }

    /// todo 这里可能可以优化一下
    pub fn d_get_mut_or_insert_with<F: FnOnce() -> Value>(
        &mut self,
        key: KeyType,
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
    type Target = HashMap<KeyType, Value>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Dict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
