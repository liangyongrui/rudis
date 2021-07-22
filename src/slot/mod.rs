//! slot 层
//!
//! 实现内部的命令 不需要兼容redis
//! slot 层的操作，加锁的时间粒度降为最小

use std::{
    borrow::BorrowMut,
    sync::{atomic::AtomicU64, Arc},
};

use parking_lot::RwLock;

use self::{cmd::WriteResp, data_type::SimpleType, dict::Dict};
use crate::{db2::BgTask, slot::cmd::Write};

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

    async fn call_write<T, C: Write<T>>(&self, cmd: C) -> crate::Result<T> {
        let id = self.next_id();
        // todo 转发请求服务
        let WriteResp {
            new_expires_at,
            payload,
        } = cmd.apply(id, self.dict.write().borrow_mut())?;
        if let Some(ea) = new_expires_at {
            // todo 发送到过期task
        }
        Ok(payload)
    }
}

/// 各种命令
impl Slot {
    pub async fn set(&self, cmd: cmd::set::Set) -> crate::Result<SimpleType> {
        self.call_write(cmd).await
    }
}
