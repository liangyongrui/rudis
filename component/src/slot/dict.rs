use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use super::data_type::DataType;
use crate::utils::now_timestamp_ms;
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Dict {
    pub write_id: u64,
    pub inner: HashMap<Arc<[u8]>, Value>,
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
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn next_id(&mut self) -> u64 {
        self.write_id += 1;
        self.write_id
    }

    pub fn last_write_op_id(&self) -> u64 {
        self.write_id
    }

    #[inline]
    pub fn d_exists(&self, key: &[u8]) -> bool {
        self.d_get(key).is_some()
    }

    #[inline]
    pub fn d_get(&self, key: &[u8]) -> Option<&Value> {
        self.get(key)
            .filter(|v| v.expires_at == 0 || v.expires_at > now_timestamp_ms())
    }

    #[inline]
    pub fn d_get_mut(&mut self, key: &[u8]) -> Option<&mut Value> {
        self.get_mut(key)
            .filter(|v| v.expires_at == 0 || v.expires_at > now_timestamp_ms())
    }

    /// todo 这里可能可以优化一下
    pub fn d_get_mut_or_insert_with<F: FnOnce() -> Value>(
        &mut self,
        key: &Arc<[u8]>,
        f: F,
    ) -> &mut Value {
        match self.entry(key.clone()) {
            std::collections::hash_map::Entry::Occupied(mut o) => {
                let expires_at = o.get().expires_at;
                if expires_at > 0 && expires_at <= now_timestamp_ms() {
                    o.insert(f());
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(f());
            }
        }
        self.get_mut(key).unwrap()
    }
}

impl Deref for Dict {
    type Target = HashMap<Arc<[u8]>, Value>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Dict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod test {
    use std::{
        alloc::Layout,
        sync::{atomic::AtomicU32, Arc},
    };

    use super::Value;

    #[test]
    fn test() {
        dbg!(Layout::new::<(Arc<[u8]>, Value)>());
        dbg!(Layout::new::<(Arc<[u8]>, Value)>());
        dbg!(Layout::new::<Box<(Arc<[u8]>, Value)>>());
        dbg!(Layout::new::<Arc<u8>>());
        dbg!(Layout::new::<(*mut u8, AtomicU32, u32)>());
        dbg!(Layout::new::<(*mut u8, u32, u32)>());
        dbg!(Layout::new::<(&[u8], u32)>());
    }
}
