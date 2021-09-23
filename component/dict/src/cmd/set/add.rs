use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::{DataType, Set},
    Dict, Value,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    pub members: Vec<Box<[u8]>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Resp {
    /// 原来的大小
    pub old_len: usize,
    /// 更新后大小
    pub new_len: usize,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::SetAdd(req)
    }
}
impl<D: Dict> Write<Resp, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Resp> {
        let old = dict.get_mut_or_insert_with(self.key, || Value {
            data: DataType::Set(Box::new(Set::new())),
            expires_at: 0,
            last_visit_time: 0,
        });
        if let DataType::Set(ref mut set) = old.data {
            let old_len = set.len();
            for m in self.members {
                set.insert(m);
            }
            Ok(Resp {
                old_len,
                new_len: set.len(),
            })
        } else {
            Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
        }
    }
}
