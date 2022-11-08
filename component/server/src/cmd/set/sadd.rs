use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/sadd>
#[derive(Debug, ParseFrames, Clone)]
pub struct Sadd {
    pub key: Key,
    pub values: Vec<Box<[u8]>>,
}

impl From<Sadd> for dict::cmd::set::add::Req {
    #[inline]
    fn from(old: Sadd) -> Self {
        Self {
            key: old.key,
            members: old.values,
        }
    }
}

impl Sadd {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.set_add(self.into())?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
