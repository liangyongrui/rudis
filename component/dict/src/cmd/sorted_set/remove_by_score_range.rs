use std::ops::{Bound, RangeBounds};

use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::{sorted_set::Node, DataType, Float},
    Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    /// 这里的得分区间(小, 大)
    pub range: (Bound<Float>, Bound<Float>),
    pub rev: bool,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::SortedSetRemoveByScoreRange(req)
    }
}
impl Write<Vec<Node>> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut Dict) -> common::Result<Vec<Node>> {
        if let Some(old) = dict.get_mut(&self.key) {
            if let DataType::SortedSet(ref mut sorted_set) = old.data {
                let mut res = vec![];
                let bigger_range = super::bigger_range(self.range);
                let value_clone = sorted_set.value.clone();
                let iter = value_clone.range(bigger_range);
                if self.rev {
                    for n in iter.rev() {
                        if self.range.contains(&n.score) {
                            sorted_set.value.remove(n);
                            if let Some(n) = sorted_set.hash.remove(&n.key) {
                                res.push(n);
                            }
                        }
                    }
                } else {
                    for n in iter {
                        if self.range.contains(&n.score) {
                            sorted_set.value.remove(n);
                            if let Some(n) = sorted_set.hash.remove(&n.key) {
                                res.push(n);
                            }
                        }
                    }
                }
                Ok(res)
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            }
        } else {
            Ok(vec![])
        }
    }
}
