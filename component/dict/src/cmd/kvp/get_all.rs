use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;

use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<HashMap<Arc<[u8]>, DataType, ahash::RandomState>> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(
        self,
        dict: &RwLock<Dict>,
    ) -> common::Result<HashMap<Arc<[u8]>, DataType, ahash::RandomState>> {
        if let Some(v) = dict.read().d_get(self.key) {
            return if let DataType::Kvp(ref kvp) = v.data {
                Ok(kvp.inner.clone())
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            };
        }
        Ok(HashMap::default())
    }
}

// utest see set
