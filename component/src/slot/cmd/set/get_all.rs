use parking_lot::RwLock;
use rpds::HashTrieSetSync;

use crate::slot::{cmd::Read, data_type::DataType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<Option<HashTrieSetSync<String>>> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<Option<HashTrieSetSync<String>>> {
        if let Some(v) = dict.read().d_get(self.key) {
            if let DataType::Set(ref set) = v.data {
                return Ok(Some(*set.inner.clone()));
            } else {
                return Err("error type".into());
            }
        }
        Ok(None)
    }
}
