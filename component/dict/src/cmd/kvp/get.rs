use parking_lot::RwLock;

use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub fields: Vec<&'a str>,
}

impl<'a> Read<Vec<DataType>> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &RwLock<Dict>) -> common::Result<Vec<DataType>> {
        if let Some(v) = dict.read().d_get(self.key) {
            return if let DataType::Kvp(ref kvp) = v.data {
                Ok(self
                    .fields
                    .into_iter()
                    .map(|field| kvp.get(field).cloned().unwrap_or(DataType::Null))
                    .collect())
            } else {
                Err("error type".into())
            };
        }
        Ok(vec![DataType::Null; self.fields.len()])
    }
}
