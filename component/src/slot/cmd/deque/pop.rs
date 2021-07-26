use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd},
    data_type::{CollectionType, DataType, KeyType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: KeyType,
    pub count: usize,
    // true left, false right
    pub left: bool,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::DequePop(req)
    }
}
impl Write<Vec<SimpleType>> for Req {
    fn apply(
        self,
        _id: u64,
        dict: &mut Dict,
    ) -> crate::Result<crate::slot::cmd::WriteResp<Vec<SimpleType>>> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            if let DataType::CollectionType(CollectionType::Deque(ref mut deque)) = v.data {
                let mut res = vec![];
                let count = self.count.min(deque.len());
                if self.left {
                    for _ in 0..count {
                        res.push(deque.pop_front().unwrap())
                    }
                } else {
                    for _ in 0..count {
                        res.push(deque.pop_back().unwrap())
                    }
                }
                return Ok(crate::slot::cmd::WriteResp {
                    payload: res,
                    new_expires_at: None,
                });
            } else {
                return Err("error type".into());
            }
        }
        Ok(crate::slot::cmd::WriteResp {
            payload: vec![],
            new_expires_at: None,
        })
    }
}

// todo utest
