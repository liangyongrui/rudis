use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/smembers>
#[derive(Debug, ParseFrames)]
pub struct Smembers<'a> {
    pub key: &'a [u8],
}

impl Smembers<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.set_get_all(dict::cmd::set::get_all::Req { key: self.key })?;
        Ok(Frame::Array(
            res.into_iter()
                .map(|t| Frame::OwnedSimple(t.into()))
                .collect(),
        ))
    }
}
