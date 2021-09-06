use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/zrem
#[derive(Debug, ParseFrames, Clone)]
pub struct Zrem {
    pub key: Key,
    pub members: Vec<Box<[u8]>>,
}

impl From<Zrem> for dict::cmd::sorted_set::remove::Req {
    fn from(old: Zrem) -> Self {
        Self {
            key: old.key,
            members: old.members,
        }
    }
}

impl Zrem {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let resp = db.sorted_set_remove(self.into())?;
        Ok(Frame::Integer((resp.old_len - resp.new_len) as _))
    }
}
