use std::borrow::Borrow;

use parking_lot::RwLock;
use rpds::HashTrieSetSync;

use crate::slot::{
    cmd::Read,
    data_type::{CollectionType, DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a SimpleType,
}

impl<'a> Read<Option<HashTrieSetSync<SimpleType>>> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<Option<HashTrieSetSync<SimpleType>>> {
        self.apply_in_lock(dict.read().borrow())
    }

    fn apply_in_lock(&self, dict: &Dict) -> crate::Result<Option<HashTrieSetSync<SimpleType>>> {
        if let Some(v) = dict.d_get(self.key) {
            if let DataType::CollectionType(CollectionType::Set(ref set)) = v.data {
                return Ok(Some((*set).clone()));
            } else {
                return Err("error type".into());
            }
        }
        Ok(None)
    }
}

// todo utest