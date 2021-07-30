use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::{atomic::AtomicBool, Arc},
};

use rpds::{HashTrieMapSync, HashTrieSetSync};

use crate::{
    expire::{self, Expiration},
    forward::{self, Forward},
    hdp::HdpStatus,
    slot::{
        cmd,
        data_type::{self, SimpleType},
        dict::{self, Dict},
        Slot,
    },
};

#[derive(Clone)]
pub struct BgTask {
    // 过期task
    pub expire_sender: flume::Sender<expire::Message>,
    // 转发task
    pub forward_sender: flume::Sender<forward::Message>,
}
pub struct Db {
    /// dicts 是否 ready
    /// 只有从节点复制请求的时候需要判断
    pub dicts_ready: AtomicBool,
    pub slots: HashMap<u16, Slot>,
}

const SIZE: u16 = 1024;

impl Db {
    pub async fn new() -> Arc<Self> {
        let expiration = Expiration::new();
        let hdp = HdpStatus::new().await;
        let forward = Forward::new(hdp.as_ref().map(|t| t.tx.clone()));
        let bg_task = BgTask {
            expire_sender: expiration.tx.clone(),
            forward_sender: forward.tx.clone(),
        };
        let mut slots = HashMap::new();
        for i in 0..SIZE {
            slots
                .entry(i)
                .or_insert_with(|| Slot::new(i, bg_task.clone()));
        }
        let db = Arc::new(Self {
            slots,
            dicts_ready: AtomicBool::new(true),
        });
        expiration.listen(Arc::clone(&db));
        forward.listen();
        if let Some(hdp) = hdp {
            let db = Arc::clone(&db);
            tokio::spawn(hdp.process(db));
        }
        db
    }

    pub fn get_slot_by_id(&self, slot_id: &u16) -> &Slot {
        self.slots.get(slot_id).unwrap()
    }

    fn get_slot(&self, key: &[u8]) -> &Slot {
        // todo 更完善的分片策略
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        let i = s.finish() % SIZE as u64;
        // todo cluster move
        self.slots.get(&(i as u16)).unwrap()
    }

    pub fn update_dict(&self, slot_id: u16, dict: Dict) {
        self.slots.get(&slot_id).unwrap().update_dict(dict);
    }
}

/// cmd
impl Db {
    pub fn get(&self, cmd: cmd::simple::get::Req) -> crate::Result<SimpleType> {
        self.get_slot(cmd.key).get(cmd)
    }
    pub fn set(&self, cmd: cmd::simple::set::Req) -> crate::Result<SimpleType> {
        self.get_slot(&cmd.key).set(cmd)
    }
    pub fn del(&self, cmd: cmd::simple::del::Req) -> crate::Result<Option<dict::Value>> {
        self.get_slot(&cmd.key).del(cmd)
    }
    pub fn expire(&self, cmd: cmd::simple::expire::Req) -> crate::Result<bool> {
        self.get_slot(&cmd.key).expire(cmd)
    }
    pub fn exists(&self, cmd: cmd::simple::exists::Req) -> crate::Result<bool> {
        self.get_slot(cmd.key).exists(cmd)
    }
    pub fn incr(&self, cmd: cmd::simple::incr::Req) -> crate::Result<i64> {
        self.get_slot(&cmd.key).incr(cmd)
    }
    pub fn kvp_incr(&self, cmd: cmd::kvp::incr::Req) -> crate::Result<i64> {
        self.get_slot(&cmd.key).kvp_incr(cmd)
    }
    pub fn kvp_del(&self, cmd: cmd::kvp::del::Req) -> crate::Result<cmd::kvp::del::Resp> {
        self.get_slot(&cmd.key).kvp_del(cmd)
    }
    pub fn kvp_set(&self, cmd: cmd::kvp::set::Req) -> crate::Result<cmd::kvp::set::Resp> {
        self.get_slot(&cmd.key).kvp_set(cmd)
    }
    pub fn kvp_exists(&self, cmd: cmd::kvp::exists::Req) -> crate::Result<bool> {
        self.get_slot(cmd.key).kvp_exists(cmd)
    }
    pub fn kvp_get(&self, cmd: cmd::kvp::get::Req<'_>) -> crate::Result<SimpleType> {
        self.get_slot(cmd.key).kvp_get(cmd)
    }
    pub fn kvp_get_all(
        &self,
        cmd: cmd::kvp::get_all::Req<'_>,
    ) -> crate::Result<Option<HashTrieMapSync<SimpleType, SimpleType>>> {
        self.get_slot(cmd.key).kvp_get_all(cmd)
    }
    pub fn deque_range(&self, cmd: cmd::deque::range::Req) -> crate::Result<Vec<SimpleType>> {
        self.get_slot(cmd.key).deque_range(cmd)
    }
    pub fn deque_len(&self, cmd: cmd::deque::len::Req) -> crate::Result<usize> {
        self.get_slot(cmd.key).deque_len(cmd)
    }
    pub fn deque_push(&self, cmd: cmd::deque::push::Req) -> crate::Result<cmd::deque::push::Resp> {
        self.get_slot(&cmd.key).deque_push(cmd)
    }
    pub fn deque_pop(&self, cmd: cmd::deque::pop::Req) -> crate::Result<Vec<SimpleType>> {
        self.get_slot(&cmd.key).deque_pop(cmd)
    }
    pub fn set_add(&self, cmd: cmd::set::add::Req) -> crate::Result<cmd::set::add::Resp> {
        self.get_slot(&cmd.key).set_add(cmd)
    }
    pub fn set_remove(&self, cmd: cmd::set::remove::Req) -> crate::Result<cmd::set::remove::Resp> {
        self.get_slot(&cmd.key).set_remove(cmd)
    }
    pub fn set_get_all(
        &self,
        cmd: cmd::set::get_all::Req<'_>,
    ) -> crate::Result<Option<HashTrieSetSync<SimpleType>>> {
        self.get_slot(cmd.key).set_get_all(cmd)
    }
    pub fn set_exists(&self, cmd: cmd::set::exists::Req<'_>) -> crate::Result<bool> {
        self.get_slot(cmd.key).set_exists(cmd)
    }
    pub fn sorted_set_range_by_lex(
        &self,
        cmd: cmd::sorted_set::range_by_lex::Req<'_>,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(cmd.key).sorted_set_range_by_lex(cmd)
    }
    pub fn sorted_set_range_by_rank(
        &self,
        cmd: cmd::sorted_set::range_by_rank::Req<'_>,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(cmd.key).sorted_set_range_by_rank(cmd)
    }
    pub fn sorted_set_range_by_score(
        &self,
        cmd: cmd::sorted_set::range_by_score::Req<'_>,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(cmd.key).sorted_set_range_by_score(cmd)
    }
    pub fn sorted_set_rank(
        &self,
        cmd: cmd::sorted_set::rank::Req<'_>,
    ) -> crate::Result<Option<usize>> {
        self.get_slot(cmd.key).sorted_set_rank(cmd)
    }
    pub fn sorted_set_add(
        &self,
        cmd: cmd::sorted_set::add::Req,
    ) -> crate::Result<cmd::sorted_set::add::Resp> {
        self.get_slot(&cmd.key).sorted_set_add(cmd)
    }
    pub fn sorted_set_remove(
        &self,
        cmd: cmd::sorted_set::remove::Req,
    ) -> crate::Result<cmd::sorted_set::remove::Resp> {
        self.get_slot(&cmd.key).sorted_set_remove(cmd)
    }
    pub fn sorted_set_remove_by_lex_range(
        &self,
        cmd: cmd::sorted_set::remove_by_lex_range::Req,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(&cmd.key).sorted_set_remove_by_lex_range(cmd)
    }
    pub fn sorted_set_remove_by_rank_range(
        &self,
        cmd: cmd::sorted_set::remove_by_rank_range::Req,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(&cmd.key).sorted_set_remove_by_rank_range(cmd)
    }
    pub fn sorted_set_remove_by_score_range(
        &self,
        cmd: cmd::sorted_set::remove_by_score_range::Req,
    ) -> crate::Result<Vec<data_type::sorted_set::Node>> {
        self.get_slot(&cmd.key)
            .sorted_set_remove_by_score_range(cmd)
    }
}
