use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, utils::now_timestamp_ms, Frame};

/// https://redis.io/commands/pexpire
#[derive(Debug, Clone, ParseFrames)]
pub struct Pexpire {
    pub key: Arc<[u8]>,
    pub milliseconds: u64,
}

impl From<Pexpire> for crate::slot::cmd::simple::expire::Req {
    fn from(old: Pexpire) -> Self {
        Self {
            key: old.key,
            expires_at: now_timestamp_ms() + old.milliseconds,
        }
    }
}

impl Pexpire {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.expire(self.into())?;
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
