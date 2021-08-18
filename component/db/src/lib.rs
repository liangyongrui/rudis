mod child_process;
mod expire;
mod forward;
mod hdp;
pub mod replica;
mod slot;

use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::{atomic::AtomicBool, Arc},
};

use ahash::AHasher;
use common::config::CONFIG;
use dict::{
    cmd,
    data_type::{self, DataType},
    Dict,
};
use parking_lot::Mutex;
use tokio::sync::broadcast;

use crate::{
    expire::Expiration,
    forward::{Message, FORWARD},
    hdp::Status,
    slot::Slot,
};

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
    pub slots: HashMap<u16, Slot, ahash::RandomState>,
    pub read_only: AtomicBool,
    // 为了优雅退出，这里的角色只是存了一下连接状态
    pub role: Mutex<Role>,
}

const SIZE: u16 = 1 << 14;

impl Db {
    pub async fn new() -> Arc<Self> {
        let expiration = Expiration::new();
        let hdp = Status::new();
        let bg_task = BgTask {
            expire_sender: expiration.tx.clone(),
            forward_sender: FORWARD.tx.clone(),
        };
        let mut slots = HashMap::default();
        for i in 0..SIZE {
            slots
                .entry(i)
                .or_insert_with(|| Slot::new(i, bg_task.clone()));
        }
        let db = Arc::new(Self {
            slots,
            read_only: AtomicBool::new(CONFIG.read_only),
            role: Mutex::new(Role::Master),
        });
        if let Some(addr) = CONFIG.master_addr {
            let db = db.clone();
            tokio::task::spawn_blocking(move || replica::process(addr, &db));
        }

        expiration.listen(Arc::clone(&db));
        if let Some(hdp) = hdp {
            FORWARD.hdp_sender.swap(Some(Arc::new(hdp.tx.clone())));
            let db = Arc::clone(&db);
            tokio::spawn(hdp.process(db));
        }
        FORWARD.db.store(Some(db.clone()));
        db
    }

    pub fn get_slot_by_id(&self, slot_id: &u16) -> &Slot {
        self.slots.get(slot_id).unwrap()
    }

    fn get_slot(&self, key: &[u8]) -> &Slot {
        // todo 更完善的分片策略
        let mut s = AHasher::new_with_keys(0, 0);
        key.hash(&mut s);
        let i = s.finish() % u64::from(SIZE);
        self.slots.get(&(i as u16)).unwrap()
    }

    pub fn replace_dict(&self, slot_id: u16, dict: Dict) {
        self.slots.get(&slot_id).unwrap().replace_dict(dict);
    }
}

/// cmd
impl Db {
    pub fn get(&self, cmd: cmd::simple::get::Req) -> common::Result<DataType> {
        self.get_slot(cmd.key).get(cmd)
    }
    pub fn ttl(&self, cmd: cmd::simple::ttl::Req<'_>) -> common::Result<cmd::simple::ttl::Resp> {
        self.get_slot(cmd.key).ttl(cmd)
    }
    pub fn set(&self, cmd: cmd::simple::set::Req) -> common::Result<DataType> {
        self.get_slot(&cmd.key).set(cmd)
    }
    pub fn del(&self, cmd: cmd::simple::del::Req) -> common::Result<Option<dict::Value>> {
        self.get_slot(&cmd.key).del(cmd)
    }
    pub fn expire(&self, cmd: cmd::simple::expire::Req) -> common::Result<bool> {
        self.get_slot(&cmd.key).expire(cmd)
    }
    pub fn exists(&self, cmd: cmd::simple::exists::Req) -> common::Result<bool> {
        self.get_slot(cmd.key).exists(cmd)
    }
    pub fn incr(&self, cmd: cmd::simple::incr::Req) -> common::Result<i64> {
        self.get_slot(&cmd.key).incr(cmd)
    }
    pub fn kvp_incr(&self, cmd: cmd::kvp::incr::Req) -> common::Result<i64> {
        self.get_slot(&cmd.key).kvp_incr(cmd)
    }
    pub fn kvp_del(&self, cmd: cmd::kvp::del::Req) -> common::Result<cmd::kvp::del::Resp> {
        self.get_slot(&cmd.key).kvp_del(cmd)
    }
    pub fn kvp_set(&self, cmd: cmd::kvp::set::Req) -> common::Result<cmd::kvp::set::Resp> {
        self.get_slot(&cmd.key).kvp_set(cmd)
    }
    pub fn kvp_exists(&self, cmd: cmd::kvp::exists::Req) -> common::Result<bool> {
        self.get_slot(cmd.key).kvp_exists(cmd)
    }
    pub fn kvp_get(&self, cmd: cmd::kvp::get::Req<'_>) -> common::Result<Vec<DataType>> {
        self.get_slot(cmd.key).kvp_get(cmd)
    }
    pub fn kvp_get_all(
        &self,
        cmd: cmd::kvp::get_all::Req<'_>,
    ) -> common::Result<HashMap<Arc<[u8]>, DataType, ahash::RandomState>> {
        self.get_slot(cmd.key).kvp_get_all(cmd)
    }
    pub fn deque_range(&self, cmd: cmd::deque::range::Req) -> common::Result<Vec<DataType>> {
        self.get_slot(cmd.key).deque_range(cmd)
    }
    pub fn deque_len(&self, cmd: cmd::deque::len::Req) -> common::Result<usize> {
        self.get_slot(cmd.key).deque_len(cmd)
    }
    pub fn deque_push(&self, cmd: cmd::deque::push::Req) -> common::Result<cmd::deque::push::Resp> {
        self.get_slot(&cmd.key).deque_push(cmd)
    }
    pub fn deque_pop(&self, cmd: cmd::deque::pop::Req) -> common::Result<Vec<DataType>> {
        self.get_slot(&cmd.key).deque_pop(cmd)
    }
    pub fn set_add(&self, cmd: cmd::set::add::Req) -> common::Result<cmd::set::add::Resp> {
        self.get_slot(&cmd.key).set_add(cmd)
    }
    pub fn set_remove(&self, cmd: cmd::set::remove::Req) -> common::Result<cmd::set::remove::Resp> {
        self.get_slot(&cmd.key).set_remove(cmd)
    }
    pub fn set_get_all(
        &self,
        cmd: cmd::set::get_all::Req<'_>,
    ) -> common::Result<HashSet<String, ahash::RandomState>> {
        self.get_slot(cmd.key).set_get_all(cmd)
    }
    pub fn set_exists(&self, cmd: cmd::set::exists::Req<'_>) -> common::Result<Vec<bool>> {
        self.get_slot(cmd.key).set_exists(cmd)
    }
    pub fn sorted_set_range_by_lex(
        &self,
        cmd: cmd::sorted_set::range_by_lex::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(cmd.key).sorted_set_range_by_lex(cmd)
    }
    pub fn sorted_set_range_by_rank(
        &self,
        cmd: cmd::sorted_set::range_by_rank::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(cmd.key).sorted_set_range_by_rank(cmd)
    }
    pub fn sorted_set_range_by_score(
        &self,
        cmd: cmd::sorted_set::range_by_score::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(cmd.key).sorted_set_range_by_score(cmd)
    }
    pub fn sorted_set_rank(
        &self,
        cmd: cmd::sorted_set::rank::Req<'_>,
    ) -> common::Result<Option<usize>> {
        self.get_slot(cmd.key).sorted_set_rank(cmd)
    }
    pub fn sorted_set_add(
        &self,
        cmd: cmd::sorted_set::add::Req,
    ) -> common::Result<cmd::sorted_set::add::Resp> {
        self.get_slot(&cmd.key).sorted_set_add(cmd)
    }
    pub fn sorted_set_remove(
        &self,
        cmd: cmd::sorted_set::remove::Req,
    ) -> common::Result<cmd::sorted_set::remove::Resp> {
        self.get_slot(&cmd.key).sorted_set_remove(cmd)
    }
    pub fn sorted_set_remove_by_lex_range(
        &self,
        cmd: cmd::sorted_set::remove_by_lex_range::Req,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(&cmd.key).sorted_set_remove_by_lex_range(cmd)
    }
    pub fn sorted_set_remove_by_rank_range(
        &self,
        cmd: cmd::sorted_set::remove_by_rank_range::Req,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(&cmd.key).sorted_set_remove_by_rank_range(cmd)
    }
    pub fn sorted_set_remove_by_score_range(
        &self,
        cmd: cmd::sorted_set::remove_by_score_range::Req,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(&cmd.key)
            .sorted_set_remove_by_score_range(cmd)
    }
}

impl Db {
    pub fn process_forward(&self, Message { id, slot, cmd }: Message) {
        match cmd {
            cmd::WriteCmd::Del(req) => self.get_slot_by_id(&slot).call_expires_update(id, req),
            cmd::WriteCmd::Expire(req) => self.get_slot_by_id(&slot).call_expires_update(id, req),
            cmd::WriteCmd::Incr(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::Set(req) => self.get_slot_by_id(&slot).call_expires_update(id, req),
            cmd::WriteCmd::KvpDel(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::KvpIncr(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::KvpSet(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::DequePop(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::DequePush(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::SetAdd(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::SetRemove(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::SortedSetAdd(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::SortedSetRemove(req) => self.get_slot_by_id(&slot).call_update(id, req),
            cmd::WriteCmd::SortedSetRemoveByRankRange(req) => {
                self.get_slot_by_id(&slot).call_update(id, req)
            }
            cmd::WriteCmd::SortedSetRemoveByScoreRange(req) => {
                self.get_slot_by_id(&slot).call_update(id, req)
            }
            cmd::WriteCmd::SortedSetRemoveByLexRange(req) => {
                self.get_slot_by_id(&slot).call_update(id, req)
            }
            cmd::WriteCmd::None => (),
        }
    }
}