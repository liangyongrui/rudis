use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/llen
#[derive(Debug, ParseFrames)]
pub struct Llen {
    pub key: Box<[u8]>,
}

impl<'a> From<&'a Llen> for dict::cmd::deque::len::Req<'a> {
    fn from(old: &'a Llen) -> Self {
        Self { key: &old.key }
    }
}

impl Llen {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let i = db.deque_len((&self).into())?;
        Ok(Frame::Integer(i as _))
    }
}
