mod data;
mod slot;
mod state;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    usize,
};

use bytes::Bytes;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

pub use self::data::Data;
use self::slot::Slot;

const SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub(crate) struct Db {
    slots: Arc<Vec<Slot>>,
}

impl Db {
    /// 获取 需要操作的Slot
    fn get_slot(&self, key: &str) -> &Slot {
        // todo 更完善的分片策略
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        let i = s.finish() % SIZE as u64;
        &self.slots[i as usize]
    }

    pub(crate) fn new() -> Self {
        let mut slots = vec![];
        for _ in 0..SIZE {
            slots.push(Slot::new());
        }
        Self {
            slots: Arc::new(slots),
        }
    }

    pub(crate) fn pexpireat(&self, key: String, expires_at: DateTime<Utc>) -> bool {
        self.get_slot(&key).pexpireat(key, expires_at)
    }
    pub(crate) fn exists(&self, keys: Vec<String>) -> usize {
        keys.into_iter()
            .filter(|key| self.get_slot(key).exists(key))
            .count()
    }

    pub(crate) fn get(&self, key: &str) -> Option<Bytes> {
        self.get_slot(key).get(key)
    }
    pub(crate) fn del(&self, keys: Vec<String>) -> usize {
        keys.into_iter()
            .filter(|key| self.get_slot(key).del(key).is_some())
            .count()
    }
    pub(crate) fn set(
        &self,
        key: String,
        value: Data,
        nxxx: Option<bool>,
        expires_at: Option<DateTime<Utc>>,
        keepttl: bool,
    ) -> Option<Bytes> {
        match self
            .get_slot(&key)
            .set(key, value, nxxx, expires_at, keepttl)
        {
            Some(Data::Bytes(bytes)) => Some(bytes),
            Some(Data::Number(n)) => Some(Bytes::copy_from_slice(n.to_string().as_bytes())),
            Some(_) => todo!("报错"),
            None => None,
        }
    }

    pub(crate) fn subscribe(&self, key: String) -> broadcast::Receiver<Bytes> {
        self.get_slot(&key).subscribe(key)
    }

    pub(crate) fn publish(&self, key: &str, value: Bytes) -> usize {
        self.get_slot(&key).publish(key, value)
    }
}
