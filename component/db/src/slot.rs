//! slot 层
//!
//! 实现内部的命令 不需要兼容redis
//! slot 层的操作，加锁的时间粒度降为最小

mod replica_update;

use std::collections::{HashMap, HashSet};

use dict::{
    cmd,
    cmd::{ExpiresOp, ExpiresOpResp, Read, Write},
    data_type,
    data_type::DataType,
    Dict, MemDict, Value,
};
use parking_lot::Mutex;
use tracing::error;

use crate::{expire, forward, BgTask};

pub struct Slot {
    pub slot_id: usize,
    // None时，表示 slot not support
    pub share_status: Mutex<Option<Box<ShareStatus>>>,
    bg_task: BgTask,
}

#[derive(Default)]
pub struct ShareStatus {
    pub dict: MemDict,
}
impl Slot {
    #[inline]
    pub fn new(slot_id: usize, bg_task: BgTask) -> Self {
        Self {
            slot_id,
            share_status: Mutex::new(Some(Box::default())),
            bg_task,
        }
    }

    /// 更新整个 dict
    ///
    /// dict 中的过期数据最好提前清理一下,
    /// 如果快照复制过来的, 过期数据并不多
    #[inline]
    pub fn replace_dict(&self, dict: MemDict) {
        if let Err(e) = self
            .bg_task
            .expire_sender
            .send(expire::Message::Clear(self.slot_id))
        {
            error!("replace_dict Clear: {:?}", e);
        };
        let expires_add = dict
            .inner
            .iter()
            .filter(|(_, v)| v.expires_at > 0)
            .map(|(k, v)| expire::Entry {
                expires_at: v.expires_at,
                slot: self.slot_id,
                key: k.clone(),
            })
            .collect();
        *self.share_status.lock() = Some(Box::new(ShareStatus { dict }));
        if let Err(e) = self
            .bg_task
            .expire_sender
            .send(expire::Message::BatchAdd(expires_add))
        {
            error!("replace_dict BatchAdd: {:?}", e);
        };
    }

    #[inline]
    fn call_write<T, C: Write<T, MemDict> + Clone>(&self, cmd: C) -> common::Result<T> {
        let cc = cmd.clone();
        // 加锁执行命令
        let (res, id) = {
            let mut share_status = self.share_status.lock();
            let s = match &mut *share_status {
                Some(s) => s,
                None => return Err("slot not support".into()),
            };
            let id = s.dict.next_id();
            (cc.apply(&mut s.dict), id)
        };

        // 转发执行完成的请求
        if let Err(e) = self.bg_task.forward_sender.send(forward::Message {
            id,
            slot: self.slot_id,
            cmd: cmd.into(),
        }) {
            error!("call_write: {:?}", e);
        };

        res
    }

    #[inline]
    fn call_expires_write<T, C: ExpiresOp<T, MemDict> + Clone>(&self, cmd: C) -> common::Result<T> {
        let cc = cmd.clone();
        // 加锁执行命令
        let (res, id) = {
            let mut share_status = self.share_status.lock();
            let s = match &mut *share_status {
                Some(s) => s,
                None => return Err("slot not support".into()),
            };
            let id = s.dict.next_id();
            (cc.apply(&mut s.dict), id)
        };

        let res = match res {
            Ok(ExpiresOpResp {
                expires_status,
                payload,
            }) => {
                match expires_status {
                    cmd::ExpiresStatus::None => (),
                    cmd::ExpiresStatus::Update(u) => {
                        if u.before != u.new {
                            if let Err(e) = self.bg_task.expire_sender.send(
                                expire::Message::Update(expire::Update {
                                    status: u,
                                    slot: self.slot_id,
                                }),
                            ) {
                                error!("call_expires_write expire_sender: {:?}", e);
                            };
                        }
                    }
                }
                Ok(payload)
            }
            Err(e) => Err(e),
        };

        // 转发执行完成的请求
        if let Err(e) = self.bg_task.forward_sender.send(forward::Message {
            id,
            slot: self.slot_id,
            cmd: cmd.into(),
        }) {
            error!("call_expires_write forward_sender: {:?}", e);
        };

        res
    }

    #[inline]
    fn call_read<T, C: Read<T, MemDict> + Clone>(&self, cmd: C) -> common::Result<T> {
        match &mut *self.share_status.lock() {
            Some(s) => cmd.apply(&mut s.dict),
            None => Err("slot not support".into()),
        }
    }

    /// clean all data
    pub(crate) fn flush(&self, sync: bool) {
        let mut status = self.share_status.lock();
        if let Some(inner) = &mut *status {
            let old = std::mem::take(inner);
            drop(status);
            if !sync {
                tokio::task::spawn_blocking(|| old);
            }
        }
    }
}

/// 写命令
impl Slot {
    #[inline]
    pub fn set(&self, cmd: cmd::simple::set::Req) -> common::Result<DataType> {
        self.call_expires_write(cmd)
    }

    #[inline]
    pub fn del(&self, cmd: cmd::simple::del::Req) -> common::Result<Option<Value>> {
        self.call_expires_write(cmd)
    }

    #[inline]
    pub fn expire(&self, cmd: cmd::simple::expire::Req) -> common::Result<bool> {
        self.call_expires_write(cmd)
    }

    #[inline]
    pub fn incr(&self, cmd: cmd::simple::incr::Req) -> common::Result<i64> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn kvp_set(&self, cmd: cmd::kvp::set::Req) -> common::Result<cmd::kvp::set::Resp> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn kvp_del(&self, cmd: cmd::kvp::del::Req) -> common::Result<cmd::kvp::del::Resp> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn kvp_incr(&self, cmd: cmd::kvp::incr::Req) -> common::Result<i64> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn deque_push(&self, cmd: cmd::deque::push::Req) -> common::Result<cmd::deque::push::Resp> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn deque_pop(&self, cmd: cmd::deque::pop::Req) -> common::Result<Vec<DataType>> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn set_add(&self, cmd: cmd::set::add::Req) -> common::Result<cmd::set::add::Resp> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn set_remove(&self, cmd: cmd::set::remove::Req) -> common::Result<cmd::set::remove::Resp> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn sorted_set_add(
        &self,
        cmd: cmd::sorted_set::add::Req,
    ) -> common::Result<cmd::sorted_set::add::Resp> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn sorted_set_remove(
        &self,
        cmd: cmd::sorted_set::remove::Req,
    ) -> common::Result<cmd::sorted_set::remove::Resp> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn sorted_set_remove_by_lex_range(
        &self,
        cmd: cmd::sorted_set::remove_by_lex_range::Req,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn sorted_set_remove_by_rank_range(
        &self,
        cmd: cmd::sorted_set::remove_by_rank_range::Req,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn sorted_set_remove_by_score_range(
        &self,
        cmd: cmd::sorted_set::remove_by_score_range::Req,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.call_write(cmd)
    }

    #[inline]
    pub fn restore(&self, cmd: cmd::server::restore::Req) -> common::Result<()> {
        self.call_expires_write(cmd)
    }
}

/// 读命令
impl Slot {
    #[inline]
    pub fn get(&self, cmd: cmd::simple::get::Req<'_>) -> common::Result<DataType> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn get_last_visit_time(
        &self,
        cmd: cmd::simple::get_last_visit_time::Req<'_>,
    ) -> common::Result<Option<u64>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn get_visit_times(
        &self,
        cmd: cmd::simple::get_visit_times::Req<'_>,
    ) -> common::Result<Option<u64>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn dump(&self, cmd: cmd::server::dump::Req<'_>) -> common::Result<Option<Vec<u8>>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn ttl(&self, cmd: cmd::simple::ttl::Req<'_>) -> common::Result<cmd::simple::ttl::Resp> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn exists(&self, cmd: cmd::simple::exists::Req<'_>) -> common::Result<bool> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn kvp_exists(&self, cmd: cmd::kvp::exists::Req<'_>) -> common::Result<bool> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn kvp_get(&self, cmd: cmd::kvp::get::Req<'_>) -> common::Result<Vec<DataType>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn kvp_get_all(
        &self,
        cmd: cmd::kvp::get_all::Req<'_>,
    ) -> common::Result<HashMap<Box<[u8]>, DataType, ahash::RandomState>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn deque_len(&self, cmd: cmd::deque::len::Req<'_>) -> common::Result<usize> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn deque_range(&self, cmd: cmd::deque::range::Req<'_>) -> common::Result<Vec<DataType>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn set_exists(&self, cmd: cmd::set::exists::Req<'_>) -> common::Result<Vec<bool>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn set_get_all(
        &self,
        cmd: cmd::set::get_all::Req<'_>,
    ) -> common::Result<HashSet<Box<[u8]>, ahash::RandomState>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn sorted_set_range_by_lex(
        &self,
        cmd: cmd::sorted_set::range_by_lex::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn sorted_set_range_by_rank(
        &self,
        cmd: cmd::sorted_set::range_by_rank::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn sorted_set_range_by_score(
        &self,
        cmd: cmd::sorted_set::range_by_score::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        self.call_read(cmd)
    }

    #[inline]
    pub fn sorted_set_rank(
        &self,
        cmd: cmd::sorted_set::rank::Req<'_>,
    ) -> common::Result<Option<usize>> {
        self.call_read(cmd)
    }
}
