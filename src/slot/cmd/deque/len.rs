use std::borrow::Borrow;

use parking_lot::RwLock;

use crate::slot::{
    cmd::Read,
    data_type::{CollectionType, DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a SimpleType,
}

impl<'a> Read<usize> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<usize> {
        self.apply_in_lock(dict.read().borrow())
    }

    fn apply_in_lock(&self, dict: &Dict) -> crate::Result<usize> {
        if let Some(v) = dict.d_get(self.key) {
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