use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/sismember
#[derive(Debug, ParseFrames)]
pub struct Sismember {
    pub key: Key,
    pub value: String,
}

impl<'a> From<&'a Sismember> for dict::cmd::set::exists::Req<'a> {
    fn from(old: &'a Sismember) -> Self {
        Self {
            key: &old.key,
            fields: vec![&old.value],
        }
    }
}

impl Sismember {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.set_exists((&self).into())?;
        Ok(Frame::Integer(if res[0] { 1 } else { 0 }))
    }
}
