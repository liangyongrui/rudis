use std::sync::Arc;

use db::Db;
use rcc_macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/srem
#[derive(Debug, ParseFrames, Clone)]
pub struct Srem {
    pub key: Arc<[u8]>,
    pub values: Vec<String>,
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
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.set_remove(self.into())?;
        Ok(Frame::Integer((res.old_len - res.new_len) as _))
    }
}
