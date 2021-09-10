use std::ops::{Bound, RangeBounds};

use common::options::Limit;
use tracing::debug;

use crate::{
    cmd::Read,
    data_type::{self, DataType, Float},
    Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub range: (Bound<Float>, Bound<Float>),
    pub limit: Limit,
    /// true 大的在前， false 小的在前
    pub rev: bool,
}

impl<D: Dict> Read<Vec<data_type::sorted_set::Node>, D> for Req<'_> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &D) -> common::Result<Vec<data_type::sorted_set::Node>> {
        if let Some(value) = dict.get(self.key) {
            if let DataType::SortedSet(ref ss) = value.data {
                let value = &ss.value;
                let bigger_range = super::bigger_range(self.range);
                debug!(?bigger_range);
                let (offset, count) = super::shape_limit(self.limit, value.len());
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
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            }
        } else {
            Ok(vec![])
        }
    }
}
