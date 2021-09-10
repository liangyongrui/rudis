use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;
/// <https://redis.io/commands/decrby>
#[derive(Debug, ParseFrames)]
pub struct Decrby {
    pub key: Key,
    pub value: i64,
}

impl From<Decrby> for dict::cmd::simple::incr::Req {
    fn from(old: Decrby) -> Self {
        Self {
            key: old.key,
            value: -old.value,
        }
    }
}

impl Decrby {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = db.incr(self.into())?;
        Ok(Frame::Integer(response))
    }
}
