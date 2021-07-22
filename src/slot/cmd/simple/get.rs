use chrono::Utc;

use crate::slot::{
    cmd::Read,
    data_type::{DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Get<'a> {
    pub key: &'a SimpleType,
}

impl<'a> Read<SimpleType> for Get<'a> {
    fn apply(self, dict: &Dict) -> crate::Result<SimpleType> {
        if let Some(v) = dict.inner.get(self.key) {
            match v.expire_at {
                Some(ea) if ea <= Utc::now() => (),
                _ => {
                    if let DataType::SimpleType(ref s) = v.data {
                        return Ok(s.clone());
                    }
                }
            }
        }
        Ok(SimpleType::Null)
    }
}

// utest see set mod
