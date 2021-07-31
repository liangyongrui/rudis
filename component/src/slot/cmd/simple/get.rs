use parking_lot::RwLock;

use crate::slot::{cmd::Read, data_type::DataType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<DataType> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<DataType> {
        if let Some(v) = dict.read().d_get(self.key) {
            return Ok(v.data.clone());
        }
        Ok(DataType::Null)
    }
}

// utest see set mod
