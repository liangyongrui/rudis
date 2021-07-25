use std::ops::Bound;

use serde::{Deserialize, Serialize};

use crate::{
    slot::{
        cmd::{Write, WriteCmd, WriteResp},
        data_type::{sorted_set::Node, CollectionType, DataType, SimpleType},
        dict::Dict,
    },
    utils::BoundExt,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: SimpleType,
    /// 这里的得分区间(小, 大)
    pub range: (Bound<SimpleType>, Bound<SimpleType>),
    pub rev: bool,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::SortedSetRemoveByLexRange(req)
    }
}
impl Write<Vec<Node>> for Req {
    fn apply(self, _id: u64, dict: &mut Dict) -> crate::Result<WriteResp<Vec<Node>>> {
        if let Some(old) = dict.d_get_mut(&self.key) {
            if let DataType::CollectionType(CollectionType::SortedSet(ref mut sorted_set)) =
                old.data
            {
                let score = match sorted_set.value.first() {
                    Some(n) => n.score,
                    None => {
                        return Ok(WriteResp {
                            new_expires_at: None,
                            payload: vec![],
                        })
                    }
                };
                let range = (
                    self.range.0.map(|key| Node { key, score }),
                    self.range.1.map(|key| Node { key, score }),
                );
                let mut res = vec![];
                let value_clone = sorted_set.value.clone();
                let iter = value_clone.range(range);
                if self.rev {
                    for n in iter.rev() {
                        sorted_set.value.remove_mut(n);
                        if let Some(n) = sorted_set.hash.remove(&n.key) {
                            res.push(n)
                        }
                    }
                } else {
                    for n in iter {
                        sorted_set.value.remove_mut(n);
                        if let Some(n) = sorted_set.hash.remove(&n.key) {
                            res.push(n)
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
