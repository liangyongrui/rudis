use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/smembers>
#[derive(Debug, ParseFrames)]
pub struct Smembers {
    pub key: Box<[u8]>,
}

impl<'a> From<&'a Smembers> for dict::cmd::set::get_all::Req<'a> {
    fn from(old: &'a Smembers) -> Self {
        Self { key: &old.key }
    }
}

impl Smembers {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame<'_>> {
        let res = db.set_get_all((&self).into())?;
        Ok(Frame::Array(
            res.into_iter()
                .map(|t| Frame::OwnedSimple(t.into()))
                .collect(),
        ))
    }
}
