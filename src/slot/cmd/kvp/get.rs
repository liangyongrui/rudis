use parking_lot::RwLock;

use crate::slot::{
    cmd::Read,
    data_type::{CollectionType, DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a SimpleType,
    pub field: &'a SimpleType,
}

impl<'a> Read<SimpleType> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<SimpleType> {
        if let Some(v) = dict.read().d_get(self.key) {
            if let DataType::CollectionType(CollectionType::Kvp(ref kvp)) = v.data {
                return Ok(kvp.get(self.field).cloned().unwrap_or(SimpleType::Null));
            } else {
                return Err("error type".into());
            }
        }
        Ok(SimpleType::Null)
    }
}

// todo utest
