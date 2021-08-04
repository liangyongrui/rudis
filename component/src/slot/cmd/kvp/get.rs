use parking_lot::RwLock;

use crate::slot::{cmd::Read, data_type::DataType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub field: &'a str,
}

impl<'a> Read<DataType> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<DataType> {
        if let Some(v) = dict.read().d_get(self.key) {
            return if let DataType::Kvp(ref kvp) = v.data {
                Ok(kvp.get(self.field).cloned().unwrap_or(DataType::Null))
            } else {
                Err("error type".into())
            };
        }
        Ok(DataType::Null)
    }
}

// utest see set
