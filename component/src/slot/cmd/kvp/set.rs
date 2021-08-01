use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    slot::{
        cmd::{Write, WriteCmd},
        data_type::{DataType, Kvp},
        dict::{self, Dict},
    },
    utils::options::NxXx,
};

/// 追加entries, 如果key 不存在，插入新的再追加
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    // key-value list
    pub entries: Vec<(String, DataType)>,
    /// nx_xx 根据 kvp 的 key 决定
    pub nx_xx: NxXx,
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
        Self::KvpSet(req)
    }
}
impl Write<Resp> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<Resp> {
        let old = dict.d_get_mut_or_insert_with(self.key, || dict::Value {
            id,
            data: DataType::Kvp(Kvp::new()),
            expires_at: 0,
        });
        if let DataType::Kvp(ref mut kvp) = old.data {
            let old_len = kvp.size();
            match self.nx_xx {
                NxXx::Nx => {
                    for (k, v) in self.entries {
                        if !kvp.contains_key(&k) {
                            kvp.insert_mut(k, v)
                        }
                    }
                }
                NxXx::Xx => {
                    for (k, v) in self.entries {
                        if kvp.contains_key(&k) {
                            kvp.insert_mut(k, v)
                        }
                    }
                }
                NxXx::None => {
                    for (k, v) in self.entries {
                        kvp.insert_mut(k, v)
                    }
                }
            }
            Ok(Resp {
                old_len,
                new_len: kvp.size(),
            })
        } else {
            Err("error type".into())
        }
    }
}
