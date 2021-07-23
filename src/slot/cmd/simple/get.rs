use std::borrow::Borrow;

use parking_lot::RwLock;

use crate::slot::{
    cmd::Read,
    data_type::{DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a SimpleType,
}

impl<'a> Read<SimpleType> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<SimpleType> {
        self.apply_in_lock(dict.read().borrow())
    }

    fn apply_in_lock(&self, dict: &Dict) -> crate::Result<SimpleType> {
        if let Some(v) = dict.d_get(self.key) {
            if let DataType::SimpleType(ref s) = v.data {
                return Ok(s.clone());
            }
        }
        Ok(SimpleType::Null)
    }
}

// utest see set mod
