use parking_lot::RwLock;

use crate::slot::{
    cmd::Read,
    data_type::{CollectionType, DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a Vec<u8>,
    pub field: &'a SimpleType,
}

impl<'a> Read<bool> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<bool> {
        if let Some(v) = dict.read().d_get(self.key) {
            if let DataType::CollectionType(CollectionType::Set(ref set)) = v.data {
                return Ok(set.contains(self.field));
            } else {
                return Err("error type".into());
            }
        }
        Ok(false)
    }
}
