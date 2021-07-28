use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd, WriteResp},
    data_type::{CollectionType, DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Vec<u8>,
    pub members: Vec<SimpleType>,
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
    fn apply(self, _id: u64, dict: &mut Dict) -> crate::Result<WriteResp<Resp>> {
        if let Some(old) = dict.d_get_mut(&self.key) {
            if let DataType::CollectionType(CollectionType::SortedSet(ref mut sorted_set)) =
                old.data
            {
                let old_len = sorted_set.hash.len();
                for ref m in self.members {
                    if let Some(ref on) = sorted_set.hash.remove(m) {
                        sorted_set.value.remove_mut(on);
                    }
                }
                Ok(WriteResp {
                    new_expires_at: None,
                    payload: Resp {
                        old_len,
                        new_len: sorted_set.hash.len(),
                    },
                })
            } else {
                Err("error type".into())
            }
        } else {
            Ok(WriteResp {
                new_expires_at: None,
                payload: Resp {
                    old_len: 0,
                    new_len: 0,
                },
            })
        }
    }
}

// todo utest
