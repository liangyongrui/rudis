use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd},
    data_type::{DataType, Set},
    dict::{self, Dict},
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
        Self::SetAdd(req)
    }
}
impl Write<Resp> for Req {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut Dict) -> crate::Result<Resp> {
        let old = dict.d_get_mut_or_insert_with(self.key, || dict::Value {
            data: DataType::Set(Set::new()),
            expires_at: 0,
        });
        if let DataType::Set(ref mut set) = old.data {
            let old_len = set.size();
            for m in self.members {
                set.insert_mut(m);
            }
            Ok(Resp {
                old_len,
                new_len: set.size(),
            })
        } else {
            Err("error type".into())
        }
    }
}
