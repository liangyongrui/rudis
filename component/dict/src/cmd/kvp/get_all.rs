use std::collections::HashMap;

use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<HashMap<Box<[u8]>, DataType, ahash::RandomState>> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(
        self,
        dict: &Dict,
    ) -> common::Result<HashMap<Box<[u8]>, DataType, ahash::RandomState>> {
        if let Some(v) = dict.get(self.key) {
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
