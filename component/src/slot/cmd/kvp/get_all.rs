use std::collections::HashMap;

use parking_lot::RwLock;

use crate::slot::{cmd::Read, data_type::DataType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<HashMap<String, DataType, ahash::RandomState>> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(
        self,
        dict: &RwLock<Dict>,
    ) -> crate::Result<HashMap<std::string::String, DataType, ahash::RandomState>> {
        if let Some(v) = dict.read().d_get(self.key) {
            return if let DataType::Kvp(ref kvp) = v.data {
                Ok(kvp.inner.clone())
            } else {
                Err("error type".into())
            };
        }
        Ok(HashMap::default())
    }
}

// utest see set
