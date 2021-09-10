use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/llen>
#[derive(Debug, ParseFrames)]
pub struct Llen<'a> {
    pub key: &'a [u8],
}

impl Llen<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let i = db.deque_len(dict::cmd::deque::len::Req { key: self.key })?;
        Ok(Frame::Integer(i as _))
    }
}
