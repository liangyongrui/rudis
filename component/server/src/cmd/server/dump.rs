use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/dump>
#[derive(Debug, Clone, ParseFrames)]
pub struct Dump {
    pub key: Box<[u8]>,
}

impl Dump {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame<'_>> {
        let res = db
            .dump(dict::cmd::server::dump::Req { key: &self.key })?
            .map_or(Frame::Null, Frame::OwnedBulk);
        Ok(res)
    }
}
