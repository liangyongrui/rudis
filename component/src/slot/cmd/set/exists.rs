use parking_lot::RwLock;

use crate::slot::{cmd::Read, data_type::DataType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub field: &'a str,
}

impl<'a> Read<bool> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<bool> {
        if let Some(v) = dict.read().d_get(self.key) {
            return if let DataType::Set(ref set) = v.data {
                Ok(set.contains(self.field))
            } else {
                Err("error type".into())
            };
        }
        Ok(false)
    }
}
