use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd, WriteResp},
    data_type::SimpleType,
    dict::{Dict, Value},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: SimpleType,
}

impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Del(req)
    }
}
/// 返回 原始值
impl Write<Option<Value>> for Req {
    fn apply(self, _id: u64, dict: &mut Dict) -> crate::Result<WriteResp<Option<Value>>> {
        if dict.d_exists(&self.key) {
            Ok(WriteResp {
                payload: dict.remove(&self.key),
                new_expires_at: None,
            })
        } else {
            Ok(WriteResp {
                payload: None,
                new_expires_at: None,
            })
        }
    }
}

// todo utest
