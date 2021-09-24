use crate::{cmd::Read, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

impl<'a, D: Dict> Read<Option<u64>, D> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]
    fn apply(self, dict: &mut D) -> common::Result<Option<u64>> {
        if let Some(v) = dict.raw_get(self.key) {
            return Ok(Some(v.get_last_visit_time() * 10));
        }
        Ok(None)
    }
}
