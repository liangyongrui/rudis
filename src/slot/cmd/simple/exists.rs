use chrono::Utc;

use crate::slot::{cmd::Read, data_type::SimpleType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Exists<'a> {
    pub key: &'a SimpleType,
}

impl<'a> Read<bool> for Exists<'a> {
    fn apply(self, dict: &Dict) -> crate::Result<bool> {
        Ok(dict
            .inner
            .get(self.key)
            .filter(|v| v.expire_at.map(|x| x <= Utc::now()).is_none())
            .is_some())
    }
}

// todo utest
