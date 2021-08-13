use parking_lot::RwLock;

use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<DataType> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &RwLock<Dict>) -> common::Result<DataType> {
        if let Some(v) = dict.read().d_get(self.key) {
            return Ok(v.data.clone());
        }
        Ok(DataType::Null)
    }
}

// utest see set mod
