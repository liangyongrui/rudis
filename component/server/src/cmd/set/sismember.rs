use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/sismember>
#[derive(Debug, ParseFrames)]
pub struct Sismember<'a> {
    pub key: &'a [u8],
    pub value: &'a [u8],
}

impl Sismember<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.set_exists(dict::cmd::set::exists::Req {
            key: self.key,
            fields: vec![self.value],
        })?;
        Ok(Frame::Integer(i64::from(res[0])))
    }
}
