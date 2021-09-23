use std::ops::Bound;

use common::{options::Limit, BoundExt};

use crate::{
    cmd::Read,
    data_type::{self, sorted_set::Node, DataType},
    Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub range: (Bound<&'a [u8]>, Bound<&'a [u8]>),
    pub limit: Limit,
    /// true 大的在前， false 小的在前
    pub rev: bool,
}

impl<D: Dict> Read<Vec<data_type::sorted_set::Node>, D> for Req<'_> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Vec<data_type::sorted_set::Node>> {
        if let Some(value) = dict.get(self.key) {
            if let DataType::SortedSet(ref ss) = value.data {
                let value = &ss.value;
                let score = match value.iter().next() {
                    Some(n) => n.score,
                    None => return Ok(vec![]),
                };
                let range = (
                    self.range.0.map(|key| Node {
                        score,
                        key: key.into(),
                    }),
                    self.range.1.map(|key| Node {
                        score,
                        key: key.into(),
                    }),
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
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            }
        } else {
            Ok(vec![])
        }
    }
}
