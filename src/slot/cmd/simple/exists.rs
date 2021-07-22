use crate::slot::{cmd::Read, data_type::SimpleType, dict::Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a SimpleType,
}

impl<'a> Read<bool> for Req<'a> {
    fn apply(self, dict: &Dict) -> crate::Result<bool> {
        Ok(dict.d_exists(self.key))
    }
}
