use std::sync::Arc;

use db::Db;
use macros::ParseFrames;

use crate::Frame;
/// https://redis.io/commands/incr
#[derive(Debug, Clone, ParseFrames)]
pub struct Incr {
    pub key: Arc<[u8]>,
}

impl From<Incr> for dict::cmd::simple::incr::Req {
    fn from(old: Incr) -> Self {
        Self {
            key: old.key,
            value: 1,
        }
    }
}

impl Incr {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = db.incr(self.into())?;
        Ok(Frame::Integer(response))
    }
}
