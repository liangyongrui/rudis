use std::{borrow::Borrow, ops::Bound};

use parking_lot::RwLock;

use crate::{
    slot::{
        cmd::Read,
        data_type::{sorted_set::Node, SimpleType},
        dict::Dict,
    },
    utils::BoundExt,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a SimpleType,
    /// 这里的得分区间(小, 大)
    pub range: (Bound<SimpleType>, Bound<SimpleType>),
    //  (offset, count)
    ///  A negative `count` returns all elements from the offset.
    pub limit: Option<(usize, i64)>,
    /// true 大的在前， false 小的在前
    pub rev: bool,
}

impl Read<Vec<crate::slot::data_type::sorted_set::Node>> for Req<'_> {
    fn apply(
        self,
        dict: &RwLock<Dict>,
    ) -> crate::Result<Vec<crate::slot::data_type::sorted_set::Node>> {
        if let Some(value) = super::get_value(self.key, dict.read().borrow())? {
            let score = match value.first() {
                Some(n) => n.score,
                None => return Ok(vec![]),
            };
            let range = (
                self.range.0.map(|key| Node { key, score }),
                self.range.1.map(|key| Node { key, score }),
            );
            let (offset, count) = super::shape_limit(self.limit, value.size());
            let iter = value.range(range);
            let res = if self.rev {
                iter.rev().skip(offset).take(count).cloned().collect()
            } else {
                iter.skip(offset).take(count).cloned().collect()
            };
            Ok(res)
        } else {
            Ok(vec![])
        }
    }
}

// todo utest
