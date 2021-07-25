use std::borrow::Borrow;

use parking_lot::RwLock;

use crate::slot::{cmd::Read, data_type::SimpleType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a SimpleType,
    pub start: i64,
    pub stop: i64,
    /// `(offset, count)`
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
            let (offset, count) = super::shape_limit(self.limit, value.size());
            let (start, stop) = super::shape_rank(self.start, self.stop, value.size());
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
            Ok(res)
        } else {
            Ok(vec![])
        }
    }
}

// todo utest
