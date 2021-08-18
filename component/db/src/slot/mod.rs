//! slot 层
//!
//! 实现内部的命令 不需要兼容redis
//! slot 层的操作，加锁的时间粒度降为最小

pub mod copy_master;

use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use dict::{
    cmd,
    cmd::{ExpiresWrite, ExpiresWriteResp, Read, Write},
    data_type,
    data_type::DataType,
    Dict, Value,
};
use parking_lot::RwLock;

use crate::{expire, forward, BgTask};

pub struct Slot {
    slot_id: u16,
    pub dict: RwLock<Dict>,
    bg_task: BgTask,
}

impl Slot {
    pub fn new(slot_id: u16, bg_task: BgTask) -> Self {
        // todo load from disk
        Self {
            slot_id,
            dict: RwLock::new(Dict::new()),
            bg_task,
        }
    }

    /// 更新整个 dict
    ///
    /// dict 中的过期数据最好提前清理一下,
    /// 如果快照复制过来的, 过期数据并不多
    pub fn replace_dict(&self, dict: Dict) {
        let _ = self
            .bg_task
            .expire_sender
            .send(expire::Message::Clear(self.slot_id));
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
        *self.dict.write() = dict;
        let _ = self
            .bg_task
            .expire_sender
            .send(expire::Message::BatchAdd(expires_add));
    }

    fn call_write<T, C: Write<T> + Clone>(&self, cmd: C) -> common::Result<T> {
        let cc = cmd.clone();
        // 加锁执行命令
        let (res, id) = {
            let mut dict = self.dict.write();
            let id = dict.next_id();
            (cc.apply(dict.borrow_mut()), id)
        };

        // 转发执行完成的请求
        let _ = self.bg_task.forward_sender.send(forward::Message {
            id,
            slot: self.slot_id,
            cmd: cmd.into(),
        });

        res
    }

    fn call_expires_write<T, C: ExpiresWrite<T> + Clone>(&self, cmd: C) -> common::Result<T> {
        let cc = cmd.clone();
        // 加锁执行命令
        let (res, id) = {
            let mut dict = self.dict.write();
            let id = dict.next_id();
            (cc.apply(dict.borrow_mut()), id)
        };

        let res = match res {
            Ok(ExpiresWriteResp {
                expires_status,
                payload,
            }) => {
                match expires_status {
                    cmd::ExpiresStatus::None => (),
                    cmd::ExpiresStatus::Update(u) => {
                        if u.before != u.new {
                            let _ = self.bg_task.expire_sender.send(expire::Message::Update(
                                expire::Update {
                                    status: u,
                                    slot: self.slot_id,
                                },
                            ));
                        }
                    }
                }
                Ok(payload)
            }
            Err(e) => Err(e),
        };

        // 转发执行完成的请求
        let _ = self.bg_task.forward_sender.send(forward::Message {
            id,
            slot: self.slot_id,
            cmd: cmd.into(),
        });

        res
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
}

/// 读命令
impl Slot {
    #[inline]
    pub fn get(&self, cmd: cmd::simple::get::Req<'_>) -> common::Result<DataType> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn ttl(&self, cmd: cmd::simple::ttl::Req<'_>) -> common::Result<cmd::simple::ttl::Resp> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn exists(&self, cmd: cmd::simple::exists::Req<'_>) -> common::Result<bool> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn kvp_exists(&self, cmd: cmd::kvp::exists::Req<'_>) -> common::Result<bool> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn kvp_get(&self, cmd: cmd::kvp::get::Req<'_>) -> common::Result<Vec<DataType>> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn kvp_get_all(
        &self,
        cmd: cmd::kvp::get_all::Req<'_>,
    ) -> common::Result<HashMap<Arc<[u8]>, DataType, ahash::RandomState>> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn deque_len(&self, cmd: cmd::deque::len::Req<'_>) -> common::Result<usize> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn deque_range(&self, cmd: cmd::deque::range::Req<'_>) -> common::Result<Vec<DataType>> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn set_exists(&self, cmd: cmd::set::exists::Req<'_>) -> common::Result<Vec<bool>> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn set_get_all(
        &self,
        cmd: cmd::set::get_all::Req<'_>,
    ) -> common::Result<HashSet<String, ahash::RandomState>> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn sorted_set_range_by_lex(
        &self,
        cmd: cmd::sorted_set::range_by_lex::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn sorted_set_range_by_rank(
        &self,
        cmd: cmd::sorted_set::range_by_rank::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn sorted_set_range_by_score(
        &self,
        cmd: cmd::sorted_set::range_by_score::Req<'_>,
    ) -> common::Result<Vec<data_type::sorted_set::Node>> {
        cmd.apply(&self.dict)
    }
    #[inline]
    pub fn sorted_set_rank(
        &self,
        cmd: cmd::sorted_set::rank::Req<'_>,
    ) -> common::Result<Option<usize>> {
        cmd.apply(&self.dict)
    }
}
