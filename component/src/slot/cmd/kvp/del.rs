use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd},
    data_type::DataType,
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: Arc<[u8]>,
    pub fields: Vec<String>,
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
impl Write<Resp> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut Dict) -> crate::Result<Resp> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            return if let DataType::Kvp(ref mut kvp) = v.data {
                let old_len = kvp.size();
                for f in self.fields {
                    kvp.remove_mut(&f);
                }
                Ok(Resp {
                    old_len,
                    new_len: kvp.size(),
                })
            } else {
                Err("error type".into())
            };
        }
        Ok(Resp {
            old_len: 0,
            new_len: 0,
        })
    }
}
