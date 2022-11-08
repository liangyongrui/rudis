use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;
/// <https://redis.io/commands/incr>
#[derive(Debug, ParseFrames)]
pub struct Incr {
    pub key: Key,
}

impl From<Incr> for dict::cmd::simple::incr::Req {
    #[inline]
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
