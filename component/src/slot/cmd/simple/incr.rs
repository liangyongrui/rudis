use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd, WriteResp},
    data_type::SimpleType,
    dict::{self, Dict},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: SimpleType,
    pub value: i64,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::Incr(req)
    }
}

/// 返回 更新后的值
impl Write<i64> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<i64>> {
        if let Some(v) = dict.d_get_mut(&self.key) {
            match v.data {
                crate::slot::data_type::DataType::SimpleType(ref mut s) => {
                    let old: i64 = (&*s).try_into()?;
                    let new = old + self.value;
                    *s = SimpleType::Integer(new);
                    Ok(WriteResp {
                        new_expires_at: None,
                        payload: new,
                    })
                }
                crate::slot::data_type::DataType::CollectionType(_) => Err("error type".into()),
            }
        } else {
            dict.insert(
                self.key,
                dict::Value {
                    expire_at: None,
                    id,
                    data: self.value.into(),
                },
            );
            Ok(WriteResp {
                new_expires_at: None,
                payload: self.value,
            })
        }
    }
}

// todo utest
