pub mod data;
mod result;
mod slot;
mod state;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::{Arc, MutexGuard},
    usize,
};

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

pub use self::data::Data;
use self::{data::Bytes, result::Result, slot::Slot, state::State};

const SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub(crate) struct Db {
    slots: Arc<Vec<Slot>>,
}

impl Db {
    /// 获取 需要操作的Slot
    fn get_state(&self, key: &str) -> MutexGuard<'_, State> {
        // todo 更完善的分片策略
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        let i = s.finish() % SIZE as u64;
        self.slots[i as usize].get_state()
    }

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

    pub(crate) fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<Bytes>> {
        self.get_state(key).lrange(key, start, stop)
    }
    pub(crate) fn lpush(&self, key: String, values: Vec<Bytes>) -> Result<usize> {
        self.get_state(&key).lpush(key, values)
    }
    pub(crate) fn rpush(&self, key: String, values: Vec<Bytes>) -> Result<usize> {
        self.get_state(&key).rpush(key, values)
    }
    pub(crate) fn lpushx(&self, key: &str, values: Vec<Bytes>) -> Result<usize> {
        self.get_state(key).lpushx(key, values)
    }
    pub(crate) fn rpushx(&self, key: &str, values: Vec<Bytes>) -> Result<usize> {
        self.get_state(key).rpushx(key, values)
    }
    pub(crate) fn incr_by(&self, key: String, value: i64) -> Result<i64> {
        self.get_state(&key).incr_by(key, value)
    }

    pub(crate) fn expire_at(&self, key: String, expires_at: DateTime<Utc>) -> bool {
        self.get_state(&key).set_expires_at(key, expires_at).0
    }
    pub(crate) fn exists(&self, keys: Vec<String>) -> usize {
        keys.into_iter()
            .filter(|key| self.get_state(key).exists(key))
            .count()
    }

    pub(crate) fn get(&self, key: &str) -> Option<bytes::Bytes> {
        self.get_slot(key).get(key)
    }
    pub(crate) fn del(&self, keys: Vec<String>) -> usize {
        keys.into_iter()
            .filter(|key| self.get_state(key).remove(key).is_some())
            .count()
    }
    pub(crate) fn set(
        &self,
        key: String,
        value: Data,
        nxxx: Option<bool>,
        expires_at: Option<DateTime<Utc>>,
        keepttl: bool,
    ) -> Option<bytes::Bytes> {
        match self
            .get_slot(&key)
            .set(key, value, nxxx, expires_at, keepttl)
        {
            Some(Data::Bytes(bytes)) => Some(bytes.get_inner_clone()),
            Some(Data::Number(n)) => Some(bytes::Bytes::copy_from_slice(n.to_string().as_bytes())),
            Some(_) => todo!("报错"),
            None => None,
        }
    }

    pub(crate) fn subscribe(&self, key: String) -> broadcast::Receiver<bytes::Bytes> {
        self.get_slot(&key).subscribe(key)
    }

    pub(crate) fn publish(&self, key: &str, value: bytes::Bytes) -> usize {
        self.get_slot(&key).publish(key, value)
    }
}
