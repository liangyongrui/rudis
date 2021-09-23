use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::DataType,
    Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    pub fields: Vec<Box<[u8]>>,
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
        Self::KvpDel(req)
    }
}
impl<D: Dict> Write<Resp, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Resp> {
        if let Some(v) = dict.get(&self.key) {
            return if let DataType::Kvp(ref mut kvp) = v.data {
                let old_len = kvp.len();
                for f in self.fields {
                    kvp.remove(&f);
                }
                Ok(Resp {
                    old_len,
                    new_len: kvp.len(),
                })
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            };
        }
        Ok(Resp {
            old_len: 0,
            new_len: 0,
        })
    }
}
