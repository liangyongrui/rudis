use common::options::Limit;
use tracing::debug;

use crate::{
    cmd::Read,
    data_type::{self, DataType},
    Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub start: i64,
    pub stop: i64,
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
                let (offset, count) = super::shape_limit(self.limit, value.len());
                let (start, stop) = super::shape_rank(self.start, self.stop, value.len());
                let range = start..stop;
                let res = if self.rev {
                    value
                        .iter()
                        .rev()
                        .enumerate()
                        .filter(|(index, _)| range.contains(index))
                        .skip(offset)
                        .take(count)
                        .map(|(_, node)| node.clone())
                        .collect()
                } else {
                    value
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| range.contains(index))
                        .skip(offset)
                        .take(count)
                        .map(|(_, node)| node.clone())
                        .collect()
                };
                debug!(?res);
                Ok(res)
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            }
        } else {
            Ok(vec![])
        }
    }
}
