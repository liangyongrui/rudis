use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteResp},
    data_type::SimpleType,
    dict::Dict,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Expire {
    pub key: SimpleType,
    pub expire_at: Option<DateTime<Utc>>,
}

/// 返回 是否更新成功
impl Write<bool> for Expire {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<bool>> {
        match dict.get_mut(&self.key) {
            Some(v) if v.expire_at.map(|x| x < Utc::now()).is_none() => {
                v.id = id;
                v.expire_at = self.expire_at;
                Ok(WriteResp {
                    payload: true,
                    new_expires_at: self.expire_at,
                })
            }
            _ => Ok(WriteResp {
                payload: false,
                new_expires_at: None,
            }),
        }
    }
}

// todo utest
