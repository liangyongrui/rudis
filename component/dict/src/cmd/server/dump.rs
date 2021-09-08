use crate::{cmd::Read, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a, D: Dict> Read<Option<Vec<u8>>, D> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &D) -> common::Result<Option<Vec<u8>>> {
        let res = if let Some(t) = dict.get(self.key) {
            Some(bincode::serialize(t)?)
        } else {
            None
        };
        Ok(res)
    }
}
