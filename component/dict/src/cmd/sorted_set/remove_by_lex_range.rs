use common::{other_type::LexRange, BoundExt};
use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::{sorted_set::Node, DataType},
    Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    pub range: LexRange,
    pub rev: bool,
}

impl From<Req> for WriteCmd {
    #[inline]
    fn from(req: Req) -> Self {
        Self::SortedSetRemoveByLexRange(req)
    }
}
impl<D: Dict> Write<Vec<Node>, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Vec<Node>> {
        if let Some(old) = dict.get(&self.key) {
            if let DataType::SortedSet(ref mut sorted_set) = old.data {
                let score = match sorted_set.value.iter().next() {
                    Some(n) => n.score,
                    None => return Ok(vec![]),
                };
                let range = (
                    self.range.0.map(|key| Node { score, key }),
                    self.range.1.map(|key| Node { score, key }),
                );
                let mut res = vec![];
                let value_clone = sorted_set.value.clone();
                let iter = value_clone.range(range);
                if self.rev {
                    for n in iter.rev() {
                        sorted_set.value.remove(n);
                        if let Some(n) = sorted_set.hash.remove(&n.key) {
                            res.push(n);
                        }
                    }
                } else {
                    for n in iter {
                        sorted_set.value.remove(n);
                        if let Some(n) = sorted_set.hash.remove(&n.key) {
                            res.push(n);
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
