use db::Db;
use macros::ParseFrames;
use tracing::debug;

use crate::Frame;

/// <https://redis.io/commands/dump>
#[derive(Debug, ParseFrames)]
pub struct Dump<'a> {
    pub key: &'a [u8],
}

impl Dump<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db
            .dump(dict::cmd::server::dump::Req { key: self.key })?
            .map_or(Frame::Null, |v| {
                debug!("{:?}", v);
                Frame::OwnedBulk(v)
            });
        Ok(res)
    }
}
