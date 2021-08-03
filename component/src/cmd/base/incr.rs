use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/incr
#[derive(Debug, Clone, ParseFrames)]
pub struct Incr {
    pub key: Arc<[u8]>,
}

impl From<Incr> for crate::slot::cmd::simple::incr::Req {
    fn from(old: Incr) -> Self {
        Self {
            key: old.key,
            value: 1,
        }
    }
}

impl Incr {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.incr(self.into())?;
        Ok(Frame::Integer(response))
    }
}
