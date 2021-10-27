use crate::{cmd::Read, data_type::DataType, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
    pub fields: Vec<&'a [u8]>,
}

impl<'a, D: Dict> Read<Vec<bool>, D> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Vec<bool>> {
        if let Some(v) = dict.get(self.key) {
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
