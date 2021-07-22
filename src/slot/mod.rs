//! slot 层
//!
//! 实现内部的命令 不需要兼容redis
//! slot 层的操作，加锁的时间粒度降为最小

use std::{
    borrow::{Borrow, BorrowMut},
    sync::atomic::AtomicU64,
};

use parking_lot::RwLock;

use self::{
    cmd::WriteResp,
    data_type::SimpleType,
    dict::{Dict, Value},
};
use crate::{
    db2::BgTask,
    forward,
    slot::cmd::{Read, Write},
};

pub mod cmd;
pub mod data_type;
pub mod dict;

pub struct Slot {
    slot_id: u16,
    next_id: AtomicU64,
    dict: RwLock<Dict>,
    bg_task: BgTask,
}

impl Slot {
    pub fn new(slot_id: u16, bg_task: BgTask) -> Self {
        // todo load from disk
        Self {
            slot_id,
            next_id: AtomicU64::new(0),
            dict: RwLock::new(Dict::new()),
            bg_task,
        }
    }
    #[inline]
    fn next_id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    async fn call_write<T, C: Write<T> + Clone>(&self, cmd: C) -> crate::Result<T> {
        let id = self.next_id();
        let _ = self
            .bg_task
            .forward_sender
            .send(forward::Message {
                id,
                slot: self.slot_id,
                cmd: cmd.clone().into(),
            })
            .await;
        let mut dict = self.dict.write();
        dict.last_write_op_id = id;
        let WriteResp {
            new_expires_at,
            payload,
        } = cmd.apply(id, dict.borrow_mut())?;
        drop(dict);
        if let Some((ea, key)) = new_expires_at {
            let _ = self
                .bg_task
                .expire_sender
                .send(crate::expire::Entry {
                    expires_at: ea,
                    slot: self.slot_id,
                    id,
                    key,
                })
                .await;
        }
        Ok(payload)
    }
}

/// 写命令
impl Slot {
    pub async fn set(&self, cmd: cmd::simple::set::Req) -> crate::Result<SimpleType> {
        self.call_write(cmd).await
    }
    pub async fn del(&self, cmd: cmd::simple::del::Req) -> crate::Result<Option<Value>> {
        self.call_write(cmd).await
    }
    pub async fn expire(&self, cmd: cmd::simple::expire::Req) -> crate::Result<bool> {
        self.call_write(cmd).await
    }
    pub async fn incr(&self, cmd: cmd::simple::incr::Req) -> crate::Result<i64> {
        self.call_write(cmd).await
    }
    pub async fn kvp_set(&self, cmd: cmd::kvp::set::Req) -> crate::Result<cmd::kvp::set::Resp> {
        self.call_write(cmd).await
    }
    pub async fn kvp_del(&self, cmd: cmd::kvp::del::Req) -> crate::Result<cmd::kvp::del::Resp> {
        self.call_write(cmd).await
    }
    pub async fn kvp_incr(&self, cmd: cmd::kvp::incr::Req) -> crate::Result<i64> {
        self.call_write(cmd).await
    }
    pub async fn deque_push(
        &self,
        cmd: cmd::deque::push::Req,
    ) -> crate::Result<cmd::deque::push::Resp> {
        self.call_write(cmd).await
    }
    pub async fn deque_pop(&self, cmd: cmd::deque::pop::Req) -> crate::Result<Vec<SimpleType>> {
        self.call_write(cmd).await
    }
}

/// 读命令
impl Slot {
    pub fn get(&self, cmd: cmd::simple::get::Req<'_>) -> crate::Result<SimpleType> {
        cmd.apply(self.dict.read().borrow())
    }
    pub fn exists(&self, cmd: cmd::simple::exists::Req<'_>) -> crate::Result<bool> {
        cmd.apply(self.dict.read().borrow())
    }
    pub fn kvp_exists(&self, cmd: cmd::kvp::exists::Req<'_>) -> crate::Result<bool> {
        cmd.apply(self.dict.read().borrow())
    }
    pub fn deque_len(&self, cmd: cmd::deque::len::Req<'_>) -> crate::Result<usize> {
        cmd.apply(self.dict.read().borrow())
    }
    pub fn deque_range(&self, cmd: cmd::deque::range::Req<'_>) -> crate::Result<Vec<SimpleType>> {
        cmd.apply(self.dict.read().borrow())
    }
}
