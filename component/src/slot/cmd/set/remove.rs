use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd},
    data_type::{CollectionType, DataType},
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub members: Vec<String>,
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
        Self::SetRemove(req)
    }
}
impl Write<Resp> for Req {
    fn apply(self, _id: u64, dict: &mut Dict) -> crate::Result<Resp> {
        if let Some(old) = dict.d_get_mut(&self.key) {
            if let DataType::CollectionType(CollectionType::Set(ref mut set)) = old.data {
                let old_len = set.size();
                for ref m in self.members {
                    set.remove_mut(m);
                }
                Ok(Resp {
                    old_len,
                    new_len: set.size(),
                })
            } else {
                Err("error type".into())
            }
        } else {
            Ok(Resp {
                old_len: 0,
                new_len: 0,
            })
        }
    }
}
