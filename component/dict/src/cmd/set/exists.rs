use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub fields: Vec<&'a str>,
}

impl<'a> Read<Vec<bool>> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &Dict) -> common::Result<Vec<bool>> {
        if let Some(v) = dict.d_get(self.key) {
            return if let DataType::Set(ref set) = v.data {
                Ok(self
                    .fields
                    .into_iter()
                    .map(|field| set.contains(field))
                    .collect())
            } else {
                Err("WRONGTYPE Operation against a key holding the wrong kind of value".into())
            };
        }
        Ok(vec![false; self.fields.len()])
    }
}
