use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a> Read<DataType> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &Dict) -> common::Result<DataType> {
        if let Some(v) = dict.get(self.key) {
            return Ok(v.data.clone());
        }
        Ok(DataType::Null)
    }
}

// utest see set mod
