use std::sync::Arc;

use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/sismember
#[derive(Debug, ParseFrames)]
pub struct Sismember {
    pub key: Arc<[u8]>,
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
