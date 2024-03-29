use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;
/// <https://redis.io/commands/incrby>
#[derive(Debug, ParseFrames)]
pub struct Incrby {
    pub key: Key,
    pub value: i64,
}

impl From<Incrby> for dict::cmd::simple::incr::Req {
    #[inline]
    fn from(old: Incrby) -> Self {
        Self {
            key: old.key,
            value: old.value,
        }
    }
}
impl Incrby {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = db.incr(self.into())?;
        Ok(Frame::Integer(response))
    }
}
