use db::Db;
use macros::ParseFrames2;

use crate::Frame;

/// <https://redis.io/commands/dump>
#[derive(Debug, Clone, ParseFrames2)]
pub struct Dump<'a> {
    pub key: &'a [u8],
}

impl Dump<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db
            .dump(dict::cmd::server::dump::Req { key: self.key })?
            .map_or(Frame::Null, Frame::OwnedBulk);
        Ok(res)
    }
}
