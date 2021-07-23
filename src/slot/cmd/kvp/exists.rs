use std::borrow::Borrow;

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

impl<'a> Read<bool> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<bool> {
        if let Some(v) = dict.read().d_get(self.key) {
            if let DataType::CollectionType(CollectionType::Kvp(ref kvp)) = v.data {
                return Ok(kvp.get(self.field).is_some());
            } else {
                return Err("error type".into());
            }
        }
        Ok(false)
    }
}

// todo utest
