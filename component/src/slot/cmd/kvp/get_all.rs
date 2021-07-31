use parking_lot::RwLock;
use rpds::HashTrieMapSync;

use crate::slot::{cmd::Read, data_type::DataType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<Option<HashTrieMapSync<String, DataType>>> for Req<'a> {
    fn apply(
        self,
        dict: &RwLock<Dict>,
    ) -> crate::Result<Option<HashTrieMapSync<String, DataType>>> {
        if let Some(v) = dict.read().d_get(self.key) {
            if let DataType::Kvp(ref kvp) = v.data {
                return Ok(Some(*kvp.inner.clone()));
            } else {
                return Err("error type".into());
            }
        }
        Ok(None)
    }
}

// utest see set
