use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::Arc,
};

use tokio::sync::mpsc;

use crate::{
    expire::{self, Expiration},
    forward::{self, Forward},
    slot::{cmd, data_type::SimpleType, dict, Slot},
};

#[derive(Clone)]
pub struct BgTask {
    // 过期task
    pub expire_sender: mpsc::Sender<expire::Entry>,
    // 转发task
    pub forward_sender: mpsc::Sender<forward::Message>,
}
pub struct Db {
    _bg_task: BgTask,
    slots: HashMap<u16, Slot>,
}

const SIZE: u16 = 1024;

impl Db {
    pub fn new() -> Arc<Self> {
        let expiration = Expiration::new();
        let forward = Forward::new();
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
            _bg_task: bg_task,
            slots,
        });
        expiration.listen(Arc::clone(&db));
        forward.listen();
        db
    }
    fn get_slot(&self, key: &SimpleType) -> &Slot {
        // todo 更完善的分片策略
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        let i = s.finish() % SIZE as u64;
        // todo cluster move
        self.slots.get(&(i as u16)).unwrap()
    }
}

/// cmd
impl Db {
    pub fn get(&self, cmd: cmd::simple::get::Req) -> crate::Result<SimpleType> {
        self.get_slot(cmd.key).get(cmd)
    }
    pub async fn set(&self, cmd: cmd::simple::set::Req) -> crate::Result<SimpleType> {
        self.get_slot(&cmd.key).set(cmd).await
    }
    pub async fn del(&self, cmd: cmd::simple::del::Req) -> crate::Result<Option<dict::Value>> {
        self.get_slot(&cmd.key).del(cmd).await
    }
    pub async fn expire(&self, cmd: cmd::simple::expire::Req) -> crate::Result<bool> {
        self.get_slot(&cmd.key).expire(cmd).await
    }
    pub fn exists(&self, cmd: cmd::simple::exists::Req) -> crate::Result<bool> {
        self.get_slot(&cmd.key).exists(cmd)
    }
    pub async fn incr(&self, cmd: cmd::simple::incr::Req) -> crate::Result<i64> {
        self.get_slot(&cmd.key).incr(cmd).await
    }
    pub async fn kvp_incr(&self, cmd: cmd::kvp::incr::Req) -> crate::Result<i64> {
        self.get_slot(&cmd.key).kvp_incr(cmd).await
    }
    pub async fn kvp_del(&self, cmd: cmd::kvp::del::Req) -> crate::Result<cmd::kvp::del::Resp> {
        self.get_slot(&cmd.key).kvp_del(cmd).await
    }
    pub async fn kvp_set(&self, cmd: cmd::kvp::set::Req) -> crate::Result<cmd::kvp::set::Resp> {
        self.get_slot(&cmd.key).kvp_set(cmd).await
    }
    pub fn kvp_exists(&self, cmd: cmd::kvp::exists::Req) -> crate::Result<bool> {
        self.get_slot(cmd.key).kvp_exists(cmd)
    }
    pub fn deque_range(&self, cmd: cmd::deque::range::Req) -> crate::Result<Vec<SimpleType>> {
        self.get_slot(cmd.key).deque_range(cmd)
    }
    pub fn deque_len(&self, cmd: cmd::deque::len::Req) -> crate::Result<usize> {
        self.get_slot(cmd.key).deque_len(cmd)
    }
    pub async fn deque_push(
        &self,
        cmd: cmd::deque::push::Req,
    ) -> crate::Result<cmd::deque::push::Resp> {
        self.get_slot(&cmd.key).deque_push(cmd).await
    }
    pub async fn deque_pop(&self, cmd: cmd::deque::pop::Req) -> crate::Result<Vec<SimpleType>> {
        self.get_slot(&cmd.key).deque_pop(cmd).await
    }
}
