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
    pub members: Vec<Box<[u8]>>,
}

pub struct Resp {
    /// 原来的大小
    pub old_len: usize,
    /// 更新后大小
    pub new_len: usize,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::SortedSetRemove(req)
    }
}
impl Write<Resp> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut Dict) -> common::Result<Resp> {
        if let Some(old) = dict.get_mut(&self.key) {
            if let DataType::SortedSet(ref mut sorted_set) = old.data {
                let old_len = sorted_set.hash.len();
                for ref m in self.members {
                    if let Some(ref on) = sorted_set.hash.remove(m) {
                        sorted_set.value.remove(on);
                    }
                }
                Ok(Resp {
                    old_len,
                    new_len: sorted_set.hash.len(),
                })
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            }
        } else {
            Ok(Resp {
                old_len: 0,
                new_len: 0,
            })
        }
    }
}
