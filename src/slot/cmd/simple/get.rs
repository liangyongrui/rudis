use chrono::Utc;

use crate::slot::{
    cmd::Read,
    data_type::{DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Get<'a> {
    pub keys: Vec<&'a SimpleType>,
}
#[derive(Debug, PartialEq, Eq)]

pub struct Resp {
    pub values: Vec<SimpleType>,
}

impl<'a> Read<Resp> for Get<'a> {
    fn apply(self, dict: &Dict) -> crate::Result<Resp> {
        let values = self
            .keys
            .into_iter()
            .map(|k| {
                if let Some(v) = dict.inner.get(k) {
                    match v.expire_at {
                        Some(ea) if ea <= Utc::now() => (),
                        _ => {
                            if let DataType::SimpleType(ref s) = v.data {
                                return s.clone();
                            }
                        }
                    }
                }
                SimpleType::Null
            })
            .collect();
        Ok(Resp { values })
    }
}

// utest see set mod
