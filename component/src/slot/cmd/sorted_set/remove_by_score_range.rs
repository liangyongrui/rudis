use std::ops::{Bound, RangeBounds};

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd, WriteResp},
    data_type::{sorted_set::Node, CollectionType, DataType, Float, KeyType},
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: KeyType,
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
    fn apply(self, _id: u64, dict: &mut Dict) -> crate::Result<WriteResp<Vec<Node>>> {
        if let Some(old) = dict.d_get_mut(&self.key) {
            if let DataType::CollectionType(CollectionType::SortedSet(ref mut sorted_set)) =
                old.data
            {
                let mut res = vec![];
                let bigger_range = super::bigger_range(self.range);
                let value_clone = sorted_set.value.clone();
                let iter = value_clone.range(bigger_range);
                if self.rev {
                    for n in iter.rev() {
                        if self.range.contains(&n.score) {
                            sorted_set.value.remove_mut(n);
                            if let Some(n) = sorted_set.hash.remove(&n.key) {
                                res.push(n)
                            }
                        }
                    }
                } else {
                    for n in iter {
                        if self.range.contains(&n.score) {
                            sorted_set.value.remove_mut(n);
                            if let Some(n) = sorted_set.hash.remove(&n.key) {
                                res.push(n)
                            }
                        }
                    }
                }
                Ok(WriteResp {
                    new_expires_at: None,
                    payload: res,
                })
            } else {
                Err("error type".into())
            }
        } else {
            Ok(WriteResp {
                new_expires_at: None,
                payload: vec![],
            })
        }
    }
}

// todo utest
