use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, utils::now_timestamp_ms, Frame};

/// https://redis.io/commands/expire
#[derive(Debug, Clone, ParseFrames)]
pub struct Expire {
    pub key: Arc<[u8]>,
    pub seconds: u64,
}

impl From<Expire> for crate::slot::cmd::simple::expire::Req {
    fn from(old: Expire) -> Self {
        Self {
            key: old.key,
            expires_at: now_timestamp_ms() + old.seconds * 1000,
        }
    }
}

impl Expire {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.expire(self.into())?;
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
