use parking_lot::RwLock;

use crate::slot::{
    cmd::Read,
    data_type::{DataType, SimpleType},
    dict::Dict,
};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a Vec<u8>,
}

impl<'a> Read<SimpleType> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<SimpleType> {
        if let Some(v) = dict.read().d_get(self.key) {
            if let DataType::SimpleType(ref s) = v.data {
                return Ok(s.clone());
            }
        }
        Ok(SimpleType::Null)
    }
}

// utest see set mod
