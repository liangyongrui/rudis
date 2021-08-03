use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/pexpireat
#[derive(Debug, Clone, ParseFrames)]
pub struct Pexpireat {
    pub key: Arc<[u8]>,
    pub ms_timestamp: u64,
}
impl From<Pexpireat> for crate::slot::cmd::simple::expire::Req {
    fn from(old: Pexpireat) -> Self {
        Self {
            key: old.key,
            expires_at: old.ms_timestamp,
        }
    }
}

impl Pexpireat {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.expire(self.into())?;
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
