use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, slot, Frame};

/// https://redis.io/commands/decr
#[derive(Debug, Clone, ParseFrames)]
pub struct Decr {
    pub key: Arc<[u8]>,
}

impl From<Decr> for slot::cmd::simple::incr::Req {
    fn from(decr: Decr) -> Self {
        Self {
            key: decr.key,
            value: -1,
        }
    }
}

impl Decr {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.incr(self.into())?;
        Ok(Frame::Integer(response))
    }
}
