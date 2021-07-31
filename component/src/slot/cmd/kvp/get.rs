use parking_lot::RwLock;

use crate::slot::{
    cmd::Read,
    data_type::{CollectionType, DataType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub field: &'a str,
}

impl<'a> Read<DataType> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<DataType> {
        if let Some(v) = dict.read().d_get(self.key) {
            if let DataType::CollectionType(CollectionType::Kvp(ref kvp)) = v.data {
                return Ok(kvp.get(self.field).cloned().unwrap_or(DataType::Null));
            } else {
                return Err("error type".into());
            }
        }
        Ok(DataType::Null)
    }
}

// utest see set
