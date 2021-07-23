use std::{
    borrow::Borrow,
    ops::{Bound, RangeBounds},
};

use parking_lot::RwLock;
use rpds::RedBlackTreeSetSync;

use crate::{
    slot::{
        cmd::Read,
        data_type::{self, sorted_set::Node, CollectionType, DataType, Float, SimpleType},
        dict::Dict,
    },
    utils::BoundExt,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a SimpleType,
    /// 这里的得分区间(小, 大)
    pub range: (Bound<Float>, Bound<Float>),
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
            let bigger_range = (
                self.range.0.map(|f| Node {
                    score: f,
                    key: SimpleType::Null,
                }),
                self.range.1.map(|f| Node {
                    score: f,
                    key: SimpleType::Big,
                }),
            );
            let (offset, count) = super::shape_limit(self.limit, value.size());
            let iter = value.range(bigger_range);
            let res = if self.rev {
                iter.rev()
                    .filter(|t| self.range.contains(&t.score))
                    .skip(offset)
                    .take(count)
                    .cloned()
                    .collect()
            } else {
                iter.filter(|t| self.range.contains(&t.score))
                    .skip(offset)
                    .take(count)
                    .cloned()
                    .collect()
            };
            Ok(res)
        } else {
            Ok(vec![])
        }
    }
}

// todo utest
