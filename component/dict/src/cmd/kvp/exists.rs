use parking_lot::RwLock;

use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub field: &'a str,
}

impl<'a> Read<bool> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &RwLock<Dict>) -> common::Result<bool> {
        if let Some(v) = dict.read().d_get(self.key) {
            return if let DataType::Kvp(ref kvp) = v.data {
                Ok(kvp.get(self.field).is_some())
            } else {
                Err("error type".into())
            };
        }
        Ok(false)
    }
}
