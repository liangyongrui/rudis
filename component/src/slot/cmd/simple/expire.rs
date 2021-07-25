use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd, WriteResp},
    data_type::SimpleType,
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: SimpleType,
    pub expire_at: Option<DateTime<Utc>>,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Expire(req)
    }
}
/// 返回 是否更新成功
impl Write<bool> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<bool>> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            v.id = id;
            v.expire_at = self.expire_at;
            Ok(WriteResp {
                payload: true,
                new_expires_at: self.expire_at.map(|ea| (ea, self.key)),
            })
        } else {
            Ok(WriteResp {
                payload: false,
                new_expires_at: None,
            })
        }
    }
}

// todo utest
