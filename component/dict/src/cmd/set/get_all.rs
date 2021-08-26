use std::collections::HashSet;

use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<HashSet<String, ahash::RandomState>> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &Dict) -> common::Result<HashSet<String, ahash::RandomState>> {
        if let Some(v) = dict.d_get(self.key) {
            return if let DataType::Set(ref set) = v.data {
                Ok(set.inner.clone())
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            };
        }
        Ok(HashSet::default())
    }
}
