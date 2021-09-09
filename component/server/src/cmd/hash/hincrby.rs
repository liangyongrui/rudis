use db::Db;
use keys::Key;
use macros::ParseFrames2;

use crate::Frame;
/// <https://redis.io/commands/hincrby>
#[derive(Debug, ParseFrames2, Clone)]
pub struct Hincrby {
    pub key: Key,
    pub field: Box<[u8]>,
    pub value: i64,
}

impl Hincrby {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let i = db.kvp_incr(dict::cmd::kvp::incr::Req {
            key: self.key,
            field: self.field,
            value: self.value,
        })?;
        Ok(Frame::Integer(i))
    }
}
