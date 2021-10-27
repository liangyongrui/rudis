use std::collections::HashSet;

use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a, D: Dict> Read<HashSet<Box<[u8]>, ahash::RandomState>, D> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<HashSet<Box<[u8]>, ahash::RandomState>> {
        if let Some(v) = dict.get(self.key) {
            return if let DataType::Set(ref set) = v.data {
                Ok(set.inner.clone())
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            };
        }
        Ok(HashSet::default())
    }
}
