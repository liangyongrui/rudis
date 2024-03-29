use common::options::NxXx;
use keys::Key;
use serde::{Deserialize, Serialize};

use crate::{
    cmd::{Write, WriteCmd},
    data_type::{DataType, Kvp},
    Dict, Value,
};

/// 追加entries, 如果key 不存在，插入新的再追加
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Key,
    // key-value list
    pub entries: Vec<(Box<[u8]>, DataType)>,
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
    #[inline]
    fn from(req: Req) -> Self {
        Self::KvpSet(req)
    }
}
impl<D: Dict> Write<Resp, D> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Resp> {
        let old = dict.get_or_insert_with(self.key, || Value {
            data: DataType::Kvp(Box::new(Kvp::new())),
            expires_at: 0,
            visit_log: Value::new_visit_log(),
        });
        if let DataType::Kvp(ref mut kvp) = old.data {
            let old_len = kvp.len();
            match self.nx_xx {
                NxXx::Nx => {
                    for (k, v) in self.entries {
                        kvp.entry(k).or_insert(v);
                    }
                }
                NxXx::Xx => {
                    for (k, v) in self.entries {
                        if let Some(old_v) = kvp.get_mut(&k) {
                            *old_v = v;
                        }
                    }
                }
                NxXx::None => {
                    for (k, v) in self.entries {
                        kvp.insert(k, v);
                    }
                }
            }
            Ok(Resp {
                old_len,
                new_len: kvp.len(),
            })
        } else {
            Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
        }
    }
}
