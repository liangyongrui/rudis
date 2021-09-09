use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/hexists>
#[derive(Debug, ParseFrames)]
pub struct Hexists<'a> {
    pub key: &'a [u8],
    pub field: &'a [u8],
}

impl Hexists<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.kvp_exists(dict::cmd::kvp::exists::Req {
            key: self.key,
            field: self.field,
        })?;
        Ok(Frame::Integer(if res { 1 } else { 0 }))
    }
}
