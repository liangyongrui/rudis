use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use bytes::Bytes;

use tokio::sync::broadcast;

use self::slot::Slot;

mod shared;
mod slot;
mod state;

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

    pub(crate) fn get(&self, key: &str) -> Option<Bytes> {
        self.get_slot(key).get(key)
    }

    pub(crate) fn set(&self, key: String, value: Bytes, expire: Option<Duration>) {
        self.get_slot(&key).set(key, value, expire)
    }

    pub(crate) fn subscribe(&self, key: String) -> broadcast::Receiver<Bytes> {
        self.get_slot(&key).subscribe(key)
    }

    pub(crate) fn publish(&self, key: &str, value: Bytes) -> usize {
        self.get_slot(&key).publish(key, value)
    }
}
