pub mod data_type;
mod result;
mod slot;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    usize,
};

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

pub use self::data_type::DataType;
use self::{
    data_type::{Blob, HashEntry, SimpleType},
    result::Result,
    slot::Slot,
};

const SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub(crate) struct Db {
    slots: Arc<Vec<Slot>>,
}

impl Db {
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
    pub fn hincrby(&self, key: &str, field: String, value: i64) -> Result<i64> {
        self.get_slot(&key).hincrby(key, field, value)
    }
    pub fn hexists(&self, key: &str, field: String) -> Result<usize> {
        self.get_slot(&key).hexists(key, field)
    }
    pub(crate) fn hdel(&self, key: &str, fields: Vec<String>) -> Result<usize> {
        self.get_slot(&key).hdel(key, fields)
    }
    pub(crate) fn hsetnx(&self, key: &str, field: String, value: SimpleType) -> Result<usize> {
        self.get_slot(&key).hsetnx(key, field, value)
    }
    pub(crate) fn hget(&self, key: &str, field: String) -> Result<Option<SimpleType>> {
        self.get_slot(&key)
            .hmget(key, vec![field])
            .map(|x| x[0].clone())
    }
    pub(crate) fn hmget(&self, key: &str, fields: Vec<String>) -> Result<Vec<Option<SimpleType>>> {
        self.get_slot(&key).hmget(key, fields)
    }
    pub(crate) fn hset(&self, key: String, pairs: Vec<HashEntry>) -> Result<usize> {
        self.get_slot(&key).hset(key, pairs)
    }
    pub(crate) fn hgetall(&self, key: &str) -> Result<Vec<HashEntry>> {
        self.get_slot(key).hgetall(key)
    }
    pub(crate) fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<Blob>> {
        self.get_slot(key).lrange(key, start, stop)
    }
    pub(crate) fn lpush(&self, key: String, values: Vec<Blob>) -> Result<usize> {
        self.get_slot(&key).lpush(&key, values)
    }
    pub(crate) fn rpush(&self, key: String, values: Vec<Blob>) -> Result<usize> {
        self.get_slot(&key).rpush(key, values)
    }
    pub(crate) fn lpushx(&self, key: &str, values: Vec<Blob>) -> Result<usize> {
        self.get_slot(key).lpushx(key, values)
    }
    pub(crate) fn rpushx(&self, key: &str, values: Vec<Blob>) -> Result<usize> {
        self.get_slot(key).rpushx(key, values)
    }
    pub(crate) fn llen(&self, key: &str) -> Result<usize> {
        self.get_slot(key).llen(key)
    }
    pub(crate) fn lpop(&self, key: &str, count: usize) -> Result<Option<Vec<Blob>>> {
        self.get_slot(key).lpop(key, count)
    }
    pub(crate) fn rpop(&self, key: &str, count: usize) -> Result<Option<Vec<Blob>>> {
        self.get_slot(key).rpop(key, count)
    }
    pub(crate) fn incr_by(&self, key: String, value: i64) -> Result<i64> {
        self.get_slot(&key).incr_by(key, value)
    }

    pub(crate) fn expires_at(&self, key: String, expires_at: DateTime<Utc>) -> bool {
        self.get_slot(&key).set_expires_at(key, expires_at)
    }
    pub(crate) fn exists(&self, keys: Vec<String>) -> usize {
        keys.into_iter()
            .filter(|key| self.get_slot(key).exists(key))
            .count()
    }

    pub(crate) fn get(&self, key: &str) -> Result<Option<SimpleType>> {
        self.get_slot(key).get_simple(key)
    }
    pub(crate) fn del(&self, keys: Vec<String>) -> usize {
        keys.into_iter()
            .filter(|key| self.get_slot(key).remove(key).is_some())
            .count()
    }
    pub(crate) async fn set(
        &self,
        key: String,
        value: SimpleType,
        nxxx: Option<bool>,
        expires_at: Option<DateTime<Utc>>,
        keepttl: bool,
    ) -> Result<Option<SimpleType>> {
        self.get_slot(&key)
            .set(key, value, nxxx, expires_at, keepttl)
            .await
    }

    pub(crate) fn subscribe(&self, key: String) -> broadcast::Receiver<bytes::Bytes> {
        self.get_slot(&key).subscribe(key)
    }

    pub(crate) fn publish(&self, key: &str, value: bytes::Bytes) -> usize {
        self.get_slot(&key).publish(key, value)
    }
}
