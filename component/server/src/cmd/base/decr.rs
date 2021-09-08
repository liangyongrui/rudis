use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/decr>
#[derive(Debug, Clone, ParseFrames)]
pub struct Decr {
    pub key: Key,
}

impl From<Decr> for dict::cmd::simple::incr::Req {
    fn from(decr: Decr) -> Self {
        Self {
            key: decr.key,
            value: -1,
        }
    }
}

impl Decr {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame<'_>> {
        let response = db.incr(self.into())?;
        Ok(Frame::Integer(response))
    }
}
