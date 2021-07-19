mod aof;
pub mod data_type;
mod result;
mod slot;
// Hard drive snapshot
mod hds;

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    net::SocketAddr,
    ops::Bound,
    sync::{Arc, Mutex},
    usize,
};

use arc_swap::ArcSwap;
use chrono::{DateTime, Utc};
use rpds::HashTrieSetSync;
use tokio::sync::mpsc;

pub use self::data_type::{DataType, SortedSetNode, ZrangeItem};
use self::{
    aof::Aof,
    data_type::{SimpleType, SimpleTypePair},
    hds::HdsStatus,
    result::Result,
    slot::Slot,
};
use crate::{
    cmd::WriteCmd,
    replica,
    utils::options::{GtLt, NxXx},
};

const SIZE: u16 = 1024;

#[derive(Debug)]
pub enum Role {
    Master(Vec<SocketAddr>),
    Replica(Option<SocketAddr>),
}

#[derive(Debug)]
pub struct Db {
    pub aof_sender: Option<mpsc::Sender<WriteCmd>>,
    role: Mutex<Role>,
    slots: Arc<HashMap<u16, Slot>>,
    hds_status: ArcSwap<HdsStatus>,
}

impl Db {
    fn get_slot(&self, key: &SimpleType) -> &Slot {
        // todo 更完善的分片策略
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        let i = s.finish() % SIZE as u64;
        // todo cluster move
        self.slots.get(&(i as u16)).unwrap()
    }

    pub fn new(role: Role) -> Arc<Self> {
        let mut slots = hds::load_slots();
        for i in 0..SIZE {
            slots.entry(i).or_insert_with(Slot::new);
        }
        let s = Arc::new(Self {
            aof_sender: Aof::start(),
            role: Mutex::new(role),
            hds_status: ArcSwap::from_pointee(HdsStatus::new()),
            slots: Arc::new(slots),
        });
        tokio::spawn(hds::run_bg_save_task(Arc::clone(&s)));
        if let Role::Replica(Some(master_addr)) = *s.role.lock().unwrap() {
            replica::update_master(master_addr)
        }
        s
    }
    pub fn zremrange_by_rank(&self, key: &SimpleType, range: (i64, i64)) -> Result<usize> {
        self.get_slot(&key).zremrange_by_rank(key, range)
    }
    pub fn zremrange_by_score(
        &self,
        key: &SimpleType,
        range: (Bound<f64>, Bound<f64>),
    ) -> Result<usize> {
        self.get_slot(&key).zremrange_by_score(key, range)
    }

    pub fn zrem(&self, key: &SimpleType, members: Vec<SimpleType>) -> Result<usize> {
        self.get_slot(&key).zrem(key, members)
    }
    pub fn zrank(&self, key: &SimpleType, member: &SimpleType, rev: bool) -> Result<Option<usize>> {
        self.get_slot(&key).zrank(key, member, rev)
    }
    pub fn zrange(
        &self,
        key: &SimpleType,
        range: ZrangeItem,
        rev: bool,
        limit: Option<(i64, i64)>,
    ) -> Result<Vec<SortedSetNode>> {
        self.get_slot(&key).zrange(key, range, rev, limit)
    }
    pub fn zadd(
        &self,
        key: SimpleType,
        values: Vec<SortedSetNode>,
        nx_xx: NxXx,
        gt_lt: GtLt,
        ch: bool,
        incr: bool,
    ) -> Result<usize> {
        self.get_slot(&key)
            .zadd(key, values, nx_xx, gt_lt, ch, incr)
    }
    pub fn smembers(&self, key: &SimpleType) -> Result<HashTrieSetSync<SimpleType>> {
        self.get_slot(&key).smembers(key)
    }
    pub fn srem(&self, key: &SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        self.get_slot(&key).srem(key, values)
    }
    pub fn sismember(&self, key: &SimpleType, value: SimpleType) -> Result<bool> {
        self.get_slot(&key)
            .smismember(key, vec![value])
            .map(|t| t[0])
    }
    pub fn smismember(&self, key: &SimpleType, values: Vec<SimpleType>) -> Result<Vec<bool>> {
        self.get_slot(&key).smismember(key, values)
    }
    pub fn sadd(&self, key: SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        self.get_slot(&key).sadd(key, values)
    }
    pub fn hincrby(&self, key: SimpleType, field: SimpleType, value: i64) -> Result<i64> {
        self.get_slot(&key).hincrby(key, field, value)
    }
    pub fn hexists(&self, key: &SimpleType, field: SimpleType) -> Result<usize> {
        self.get_slot(&key).hexists(key, field)
    }
    pub fn hdel(&self, key: &SimpleType, fields: Vec<SimpleType>) -> Result<usize> {
        self.get_slot(&key).hdel(key, fields)
    }
    pub fn hsetnx(&self, key: SimpleType, field: SimpleType, value: SimpleType) -> Result<usize> {
        self.get_slot(&key).hsetnx(key, field, value)
    }
    pub fn hget(&self, key: &SimpleType, field: SimpleType) -> Result<Option<SimpleType>> {
        self.get_slot(&key)
            .hmget(key, vec![field])
            .map(|x| x[0].clone())
    }
    pub fn hmget(
        &self,
        key: &SimpleType,
        fields: Vec<SimpleType>,
    ) -> Result<Vec<Option<SimpleType>>> {
        self.get_slot(&key).hmget(key, fields)
    }
    pub fn hset(&self, key: SimpleType, pairs: Vec<SimpleTypePair>) -> Result<usize> {
        self.get_slot(&key).hset(key, pairs)
    }
    pub fn hgetall(&self, key: &SimpleType) -> Result<Vec<SimpleTypePair>> {
        self.get_slot(key).hgetall(key)
    }
    pub fn lrange(&self, key: &SimpleType, start: i64, stop: i64) -> Result<Vec<SimpleType>> {
        self.get_slot(key).lrange(key, start, stop)
    }
    pub fn lpush(&self, key: SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        self.get_slot(&key).lpush(key, values)
    }
    pub fn rpush(&self, key: SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        self.get_slot(&key).rpush(key, values)
    }
    pub fn lpushx(&self, key: &SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        self.get_slot(key).lpushx(key, values)
    }
    pub fn rpushx(&self, key: &SimpleType, values: Vec<SimpleType>) -> Result<usize> {
        self.get_slot(key).rpushx(key, values)
    }
    pub fn llen(&self, key: &SimpleType) -> Result<usize> {
        self.get_slot(key).llen(key)
    }
    pub fn lpop(&self, key: &SimpleType, count: usize) -> Result<Option<Vec<SimpleType>>> {
        self.get_slot(key).lpop(key, count)
    }
    pub fn rpop(&self, key: &SimpleType, count: usize) -> Result<Option<Vec<SimpleType>>> {
        self.get_slot(key).rpop(key, count)
    }

    pub fn incr_by(&self, key: SimpleType, value: i64) -> Result<i64> {
        self.get_slot(&key).incr_by(key, value)
    }

    pub async fn expires_at(&self, key: &SimpleType, expires_at: DateTime<Utc>) -> bool {
        self.get_slot(&key).set_expires_at(key, expires_at).await
    }
    pub fn exists(&self, keys: Vec<SimpleType>) -> usize {
        keys.into_iter()
            .filter(|key| self.get_slot(key).exists(key))
            .count()
    }

    pub fn get(&self, key: &SimpleType) -> Result<Option<SimpleType>> {
        self.get_slot(key).get_simple(key)
    }
    pub fn del(&self, keys: Vec<SimpleType>) -> usize {
        keys.into_iter()
            .filter(|key| self.get_slot(key).remove(key).is_some())
            .count()
    }
    pub async fn set(
        &self,
        key: SimpleType,
        value: SimpleType,
        nx_xx: NxXx,
        expires_at: Option<DateTime<Utc>>,
        keepttl: bool,
    ) -> Result<Option<SimpleType>> {
        self.get_slot(&key)
            .set(key, value, nx_xx, expires_at, keepttl)
            .await
    }

    pub fn bgsave(&self) {
        // todo 防止多个save执行
        // self.bg_save_run.compare_exchange(current, new, success, failure)
        hds::save_slots(self.slots.as_ref())
    }

    pub fn replicaof(&self, master_addr: SocketAddr) {
        replica::update_master(master_addr);
        let mut role = self.role.lock().unwrap();
        *role = Role::Replica(Some(master_addr));
    }
}
