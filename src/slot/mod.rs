//! slot 层
//!
//! 实现内部的命令 不需要兼容redis
//! slot 层的操作，加锁的时间粒度降为最小

use std::{
    borrow::BorrowMut,
    sync::{atomic::AtomicU64, Arc},
};

use parking_lot::RwLock;

use self::dict::Dict;
use crate::slot::cmd::Write;

pub mod cmd;
pub mod data_type;
pub mod dict;

pub struct Slot {
    next_id: AtomicU64,
    dict: Arc<RwLock<Dict>>,
}

impl Slot {
    #[inline]
    fn next_id(&self) -> u64 {
        let res = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        res
    }
}

impl Slot {
    pub async fn set(&self, cmd: cmd::set::Set) -> crate::Result<cmd::set::Resp> {
        let id = self.next_id();
        // todo 转发请求服务
        cmd.apply(id, self.dict.write().borrow_mut())
        // todo 发送到过期task
    }
}
