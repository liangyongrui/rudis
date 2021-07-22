use std::convert::TryInto;

use serde::{Deserialize, Serialize};

use crate::slot::{
    cmd::{Write, WriteCmd, WriteResp},
    data_type::{CollectionType, DataType, Kvp, SimpleType},
    dict::{self, Dict},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Req {
    pub key: SimpleType,
    pub field: SimpleType,
    pub value: i64,
}
impl From<Req> for WriteCmd {
    fn from(req: Req) -> Self {
        Self::KvpIncr(req)
    }
}

/// 返回 更新后的值
impl Write<i64> for Req {
    fn apply(self, id: u64, dict: &mut Dict) -> crate::Result<WriteResp<i64>> {
        let v = dict.d_get_mut_or_insert_with(self.key, || dict::Value {
            expire_at: None,
            id,
            data: DataType::CollectionType(CollectionType::Kvp(Kvp::new())),
        });
        match v.data {
            crate::slot::data_type::DataType::CollectionType(CollectionType::Kvp(ref mut kvp)) => {
                if let Some(s) = kvp.get_mut(&self.field) {
                    let old: i64 = (&*s).try_into()?;
                    let new = old + self.value;
                    *s = SimpleType::Integer(new);
                    Ok(WriteResp {
                        new_expires_at: None,
                        payload: new,
                    })
                } else {
                    kvp.insert_mut(self.field, SimpleType::Integer(self.value));
                    Ok(WriteResp {
                        new_expires_at: None,
                        payload: self.value,
                    })
                }
            }
            _ => Err("error type".into()),
        }
    }
}

// todo utest
