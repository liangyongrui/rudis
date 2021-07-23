use std::borrow::Borrow;

use parking_lot::RwLock;

use crate::slot::{cmd::Read, data_type::SimpleType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a SimpleType,
}

impl<'a> Read<bool> for Req<'a> {
    fn apply(self, dict: &RwLock<Dict>) -> crate::Result<bool> {
        Ok(dict.read().d_exists(self.key))
    }
}
