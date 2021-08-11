use std::ops::Bound;

use parking_lot::RwLock;

use crate::{
    slot::{
        cmd::Read,
        data_type::{sorted_set::Node, DataType},
        dict::Dict,
    },
    utils::BoundExt,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    /// 这里的得分区间(小, 大)
    pub range: (Bound<String>, Bound<String>),
    //  (offset, count)
    ///  A negative `count` returns all elements from the offset.
    pub limit: Option<(usize, i64)>,
    /// true 大的在前， false 小的在前
    pub rev: bool,
}

impl Read<Vec<crate::slot::data_type::sorted_set::Node>> for Req<'_> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(
        self,
        dict: &RwLock<Dict>,
    ) -> crate::Result<Vec<crate::slot::data_type::sorted_set::Node>> {
        if let Some(value) = dict.read().d_get(self.key) {
            if let DataType::SortedSet(ref ss) = value.data {
                let value = ss.value.as_ref();
                let score = match value.iter().next() {
                    Some(n) => n.score,
                    None => return Ok(vec![]),
                };
                let range = (
                    self.range.0.map(|key| Node { score, key }),
                    self.range.1.map(|key| Node { score, key }),
                );
                let (offset, count) = super::shape_limit(self.limit, value.len());
                let iter = value.range(range);
                let res = if self.rev {
                    iter.rev().skip(offset).take(count).cloned().collect()
                } else {
                    iter.skip(offset).take(count).cloned().collect()
                };
                Ok(res)
            } else {
                Err("error type".into())
            }
        } else {
            Ok(vec![])
        }
    }
}

// todo utest
