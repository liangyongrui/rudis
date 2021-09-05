#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::doc_markdown)]
#![allow(unstable_name_collisions)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::let_underscore_drop)]

pub mod child_process;
mod expire;
mod forward;
mod pd_handle;
mod slot;

use std::{
    collections::{HashMap, HashSet},
    process::exit,
    sync::{atomic::AtomicBool, Arc},
};

use common::{config::CONFIG, SLOT_SIZE};
use crc::Crc;
use dict::{
    cmd,
    data_type::{self, DataType},
    Dict,
};
use forward::Forward;
use keys::Key;
use parking_lot::Mutex;
use tokio::sync::broadcast;
use tracing::error;

use crate::{expire::Expiration, slot::Slot};

#[derive(Clone)]
pub struct BgTask {
    // 过期task
    pub expire_sender: flume::Sender<expire::Message>,
    // 转发task
    pub forward_sender: flume::Sender<forward::Message>,
}

pub enum Role {
    Master,
    Replica(broadcast::Sender<()>),
}

pub struct Db {
    pub slots: Vec<Slot>,
    pub read_only: AtomicBool,
    // 为了优雅退出，这里的角色只是存了一下连接状态
    pub role: Mutex<Role>,
}

// https://redis.io/topics/cluster-spec
const SIZE_MOD: u16 = 16383;
const CRC_HASH: Crc<u16> = Crc::<u16>::new(&crc::CRC_16_XMODEM);

impl Db {
    pub async fn new() -> Arc<Self> {
        let expiration = Expiration::new();
        let forward = Forward::new();

        let bg_task = BgTask {
            expire_sender: expiration.tx.clone(),
            forward_sender: forward.tx.clone(),
        };
        let mut slots = Vec::with_capacity(SLOT_SIZE);
        for i in 0..SLOT_SIZE {
            slots.push(Slot::new(i, bg_task.clone()));
        }
        let db = Arc::new(Self {
            slots,
            read_only: AtomicBool::new(false),
            role: Mutex::new(Role::Master),
        });

        if let Some(pd) = CONFIG.from_pd {
            if let Err(e) = pd_handle::run(db.clone(), pd).await {
                error!("{:?}", e);
                exit(-1)
            };
        }

        expiration.listen(Arc::clone(&db));
        forward.listen();
        db
    }

    #[inline]
    pub fn get_slot_by_id(&self, slot_id: usize) -> &Slot {
        &self.slots[slot_id]
    }

    #[inline]
    fn get_slot(&self, mut key: &[u8]) -> &Slot {
        if let Some(begin) = key.iter().position(|t| *t == b'{') {
            if let Some(end) = key[begin..].iter().position(|t| *t == b'}') {
                key = &key[begin + 1..begin + end];
            }
        }
        let i = CRC_HASH.checksum(key) & SIZE_MOD;
        &self.slots[i as usize]
    }

    #[inline]
    pub fn replace_dict(&self, slot_id: usize, dict: Dict) {
        self.slots[slot_id].replace_dict(dict);
    }
}

/// cmd
impl Db {
    #[inline]
    pub fn get(&self, cmd: cmd::simple::get::Req) -> common::Result<DataType> {
        self.get_slot(cmd.key).get(cmd)
    }
    #[inline]
    pub fn ttl(&self, cmd: cmd::simple::ttl::Req<'_>) -> common::Result<cmd::simple::ttl::Resp> {
        self.get_slot(cmd.key).ttl(cmd)
    }
    #[inline]
    pub fn set(&self, cmd: cmd::simple::set::Req) -> common::Result<DataType> {
        self.get_slot(&cmd.key).set(cmd)
    }
    #[inline]
    pub fn del(&self, cmd: cmd::simple::del::Req) -> common::Result<Option<dict::Value>> {
        self.get_slot(&cmd.key).del(cmd)
    }
    #[inline]
    pub fn expire(&self, cmd: cmd::simple::expire::Req) -> common::Result<bool> {
        self.get_slot(&cmd.key).expire(cmd)
    }
    #[inline]
    pub fn exists(&self, cmd: cmd::simple::exists::Req) -> common::Result<bool> {
        self.get_slot(cmd.key).exists(cmd)
    }
    #[inline]
    pub fn incr(&self, cmd: cmd::simple::incr::Req) -> common::Result<i64> {
        self.get_slot(&cmd.key).incr(cmd)
    }
    #[inline]
    pub fn kvp_incr(&self, cmd: cmd::kvp::incr::Req) -> common::Result<i64> {
        self.get_slot(&cmd.key).kvp_incr(cmd)
    }
    #[inline]
    pub fn kvp_del(&self, cmd: cmd::kvp::del::Req) -> common::Result<cmd::kvp::del::Resp> {
        self.get_slot(&cmd.key).kvp_del(cmd)
    }
    #[inline]
    pub fn kvp_set(&self, cmd: cmd::kvp::set::Req) -> common::Result<cmd::kvp::set::Resp> {
        self.get_slot(&cmd.key).kvp_set(cmd)
    }
    #[inline]
    pub fn kvp_exists(&self, cmd: cmd::kvp::exists::Req) -> common::Result<bool> {
        self.get_slot(cmd.key).kvp_exists(cmd)
    }
    #[inline]
    pub fn kvp_get(&self, cmd: cmd::kvp::get::Req<'_>) -> common::Result<Vec<DataType>> {
        self.get_slot(cmd.key).kvp_get(cmd)
    }
    #[inline]
    pub fn kvp_get_all(
        &self,
        cmd: cmd::kvp::get_all::Req<'_>,
    ) -> common::Result<HashMap<Key, DataType, ahash::RandomState>> {
        self.get_slot(cmd.key).kvp_get_all(cmd)
    }
    #[inline]
    pub fn deque_range(&self, cmd: cmd::deque::range::Req) -> common::Result<Vec<DataType>> {
        self.get_slot(cmd.key).deque_range(cmd)
    }
    #[inline]
    pub fn deque_len(&self, cmd: cmd::deque::len::Req) -> common::Result<usize> {
        self.get_slot(cmd.key).deque_len(cmd)
    }
    #[inline]
    pub fn deque_push(&self, cmd: cmd::deque::push::Req) -> common::Result<cmd::deque::push::Resp> {
        self.get_slot(&cmd.key).deque_push(cmd)
    }
    #[inline]
    pub fn deque_pop(&self, cmd: cmd::deque::pop::Req) -> common::Result<Vec<DataType>> {
        self.get_slot(&cmd.key).deque_pop(cmd)
    }
    #[inline]
    pub fn set_add(&self, cmd: cmd::set::add::Req) -> common::Result<cmd::set::add::Resp> {
        self.get_slot(&cmd.key).set_add(cmd)
    }
    #[inline]
    pub fn set_remove(&self, cmd: cmd::set::remove::Req) -> common::Result<cmd::set::remove::Resp> {
        self.get_slot(&cmd.key).set_remove(cmd)
    }
    #[inline]
    pub fn set_get_all(
        &self,
        cmd: cmd::set::get_all::Req<'_>,
    ) -> common::Result<HashSet<String, ahash::RandomState>> {
        self.get_slot(cmd.key).set_get_all(cmd)
    }
    #[inline]
    pub fn set_exists(&self, cmd: cmd::set::exists::Req<'_>) -> common::Result<Vec<bool>> {
        self.get_slot(cmd.key).set_exists(cmd)
    }
    #[inline]
    pub fn sorted_set_range_by_lex(
        &self,
        cmd: cmd::sorted_set::range_by_lex::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(cmd.key).sorted_set_range_by_lex(cmd)
    }
    #[inline]
    pub fn sorted_set_range_by_rank(
        &self,
        cmd: cmd::sorted_set::range_by_rank::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(cmd.key).sorted_set_range_by_rank(cmd)
    }
    #[inline]
    pub fn sorted_set_range_by_score(
        &self,
        cmd: cmd::sorted_set::range_by_score::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(cmd.key).sorted_set_range_by_score(cmd)
    }
    #[inline]
    pub fn sorted_set_rank(
        &self,
        cmd: cmd::sorted_set::rank::Req<'_>,
    ) -> common::Result<Option<usize>> {
        self.get_slot(cmd.key).sorted_set_rank(cmd)
    }
    #[inline]
    pub fn sorted_set_add(
        &self,
        cmd: cmd::sorted_set::add::Req,
    ) -> common::Result<cmd::sorted_set::add::Resp> {
        self.get_slot(&cmd.key).sorted_set_add(cmd)
    }
    #[inline]
    pub fn sorted_set_remove(
        &self,
        cmd: cmd::sorted_set::remove::Req,
    ) -> common::Result<cmd::sorted_set::remove::Resp> {
        self.get_slot(&cmd.key).sorted_set_remove(cmd)
    }
    #[inline]
    pub fn sorted_set_remove_by_lex_range(
        &self,
        cmd: cmd::sorted_set::remove_by_lex_range::Req,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(&cmd.key).sorted_set_remove_by_lex_range(cmd)
    }
    #[inline]
    pub fn sorted_set_remove_by_rank_range(
        &self,
        cmd: cmd::sorted_set::remove_by_rank_range::Req,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(&cmd.key).sorted_set_remove_by_rank_range(cmd)
    }
    #[inline]
    pub fn sorted_set_remove_by_score_range(
        &self,
        cmd: cmd::sorted_set::remove_by_score_range::Req,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(&cmd.key)
            .sorted_set_remove_by_score_range(cmd)
    }
}
