use common::now_timestamp_ms;

use crate::{cmd::Read, Dict};

#[derive(Debug, Clone)]
pub struct Req<'a> {
    pub key: &'a [u8],
}

#[derive(Debug)]
pub enum Resp {
    ///the key exists but has no associated expire
    None,
    /// the key does not exist
    NotExist,
    ///  remaining milliseconds
    Ttl(u64),
}

impl<'a> Read<Resp> for Req<'a> {
    #[tracing::instrument(skip(dict), level = "debug")]

    fn apply(self, dict: &Dict) -> common::Result<Resp> {
        let now = now_timestamp_ms();
        let res = dict
            .get(self.key)
            .map(|v| v.expires_at)
            .filter(|&expires_at| expires_at == 0 || expires_at > now)
            .map_or(Resp::NotExist, |expires_at| {
                if expires_at == 0 {
                    Resp::None
                } else {
                    Resp::Ttl(expires_at - now)
                }
            });
        Ok(res)
    }
}
