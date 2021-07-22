use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteResp},
    data_type::SimpleType,
    dict::{self, Dict},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Incr {
    pub key: SimpleType,
    pub value: i64,
}

/// 返回 更新后的值
impl Write<i64> for Incr {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<i64>> {
        let (new, expire_at) = if let Some(v) = dict.d_get(&self.key) {
            match v.data {
                crate::slot::data_type::DataType::SimpleType(ref s) => {
                    let old: i64 = s.try_into()?;
                    (old + self.value, v.expire_at)
                }
                crate::slot::data_type::DataType::CollectionType(_) => {
                    return Err("error type".into())
                }
            }
        } else {
            (self.value, None)
        };
        dict.insert(
            self.key,
            dict::Value {
                expire_at,
                id,
                data: new.into(),
            },
        );
        Ok(WriteResp {
            new_expires_at: expire_at,
            payload: new,
        })
    }
}

// todo utest
