use std::convert::TryInto;

use chrono::Utc;
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
        let (new, ea) = match dict.entry(self.key) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                if e.get().expire_at.map(|x| x < Utc::now()).is_none() {
                    // 过期未删除
                    e.insert(dict::Value {
                        expire_at: None,
                        data: self.value.into(),
                        id,
                    });
                    (self.value, None)
                } else {
                    let new = match &mut e.get_mut().data {
                        crate::slot::data_type::DataType::SimpleType(s) => {
                            let old: i64 = (&*s).try_into()?;
                            let new = old + self.value;
                            *s = SimpleType::Integer(new);
                            new
                        }
                        crate::slot::data_type::DataType::CollectionType(_) => return Err("error type".into()),
                    };
                    (new, e.get().expire_at)
                }
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(dict::Value {
                    expire_at: None,
                    data: self.value.into(),
                    id,
                });
                (self.value, None)
            }
        };
        Ok(WriteResp {
            new_expires_at: ea,
            payload: new,
        })
    }
}

// todo utest
