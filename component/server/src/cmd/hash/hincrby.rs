use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;
/// <https://redis.io/commands/hincrby>
#[derive(Debug, ParseFrames, Clone)]
pub struct Hincrby {
    pub key: Key,
    pub field: Box<[u8]>,
    pub value: i64,
}

impl From<Hincrby> for dict::cmd::kvp::incr::Req {
    fn from(old: Hincrby) -> Self {
        Self {
            key: old.key,
            field: old.field,
            value: old.value,
        }
    }
}

impl Hincrby {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame<'_>> {
        let i = db.kvp_incr(self.into())?;
        Ok(Frame::Integer(i))
    }
}
