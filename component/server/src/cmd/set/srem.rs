use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/srem>
#[derive(Debug, ParseFrames, Clone)]
pub struct Srem {
    pub key: Key,
    pub values: Vec<Box<[u8]>>,
}

impl From<Srem> for dict::cmd::set::remove::Req {
    fn from(old: Srem) -> Self {
        Self {
            key: old.key,
            members: old.values,
        }
    }
}
impl Srem {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame<'_>> {
        let res = db.set_remove(self.into())?;
        Ok(Frame::Integer((res.old_len - res.new_len) as _))
    }
}
