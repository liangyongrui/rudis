use parking_lot::RwLock;

use crate::slot::{
    cmd::Read,
    data_type::{CollectionType, DataType, KeyType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a KeyType,
}

impl<'a> Read<usize> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<usize> {
        if let Some(v) = dict.read().d_get(self.key) {
            if let DataType::CollectionType(CollectionType::Deque(ref deque)) = v.data {
                return Ok(deque.len());
            } else {
                return Err("error type".into());
            }
        }
        Ok(0)
    }
}

// todo utest
