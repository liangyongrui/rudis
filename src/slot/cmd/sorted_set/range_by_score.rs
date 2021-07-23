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
    //  (offset,count)
    ///  A negative `count` returns all elements from the offset.
    ///  A negative `offset`, offset = 0
    pub limit: Option<(i64, i64)>,
    /// true 大的在前， false 小的在前
    pub rev: bool,
}

impl Read<Vec<crate::slot::data_type::sorted_set::Node>, Option<RedBlackTreeSetSync<Node>>>
    for Req<'_>
{
    fn apply(
        self,
        dict: &RwLock<Dict>,
    ) -> crate::Result<Vec<crate::slot::data_type::sorted_set::Node>> {
        if let Some(value) = self.apply_in_lock(dict.read().borrow())? {
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
            let (offset, count) = data_type::sorted_set::sharp_limit(self.limit, value.size());
            let iter = value.range(bigger_range);
            let res = if self.rev {
                iter.rev()
                    .filter(|t| self.range.contains(&t.score))
                    .take(count)
                    .skip(offset)
                    .cloned()
                    .collect()
            } else {
                iter.filter(|t| self.range.contains(&t.score))
                    .take(count)
                    .skip(offset)
                    .cloned()
                    .collect()
            };
            Ok(res)
        } else {
            Ok(vec![])
        }
    }

    fn apply_in_lock(&self, dict: &Dict) -> crate::Result<Option<RedBlackTreeSetSync<Node>>> {
        if let Some(v) = dict.d_get(self.key) {
            if let DataType::CollectionType(CollectionType::SortedSet(ref sorted_set)) = v.data {
                Ok(Some(sorted_set.value.clone()))
            } else {
                Err("error type".into())
            }
        } else {
            Ok(None)
        }
    }
}
