use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd},
    data_type::{sorted_set::Node, CollectionType, DataType},
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub start: i64,
    pub stop: i64,
    pub rev: bool,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::SortedSetRemoveByRankRange(req)
    }
}
impl Write<Vec<Node>> for Req {
    fn apply(self, _id: u64, dict: &mut Dict) -> crate::Result<Vec<Node>> {
        if let Some(old) = dict.d_get_mut(&self.key) {
            if let DataType::CollectionType(CollectionType::SortedSet(ref mut sorted_set)) =
                old.data
            {
                let mut res = vec![];
                let (start, stop) = super::shape_rank(self.start, self.stop, sorted_set.hash.len());
                if self.rev {
                    for (i, n) in sorted_set.value.clone().iter().enumerate().rev() {
                        if i < start {
                            continue;
                        }
                        if i >= stop {
                            break;
                        }
                        sorted_set.value.remove_mut(n);
                        if let Some(n) = sorted_set.hash.remove(&n.key) {
                            res.push(n)
                        }
                    }
                } else {
                    for (i, n) in sorted_set.value.clone().iter().enumerate() {
                        if i < start {
                            continue;
                        }
                        if i >= stop {
                            break;
                        }
                        sorted_set.value.remove_mut(n);
                        if let Some(n) = sorted_set.hash.remove(&n.key) {
                            res.push(n)
                        }
                    }
                }
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
