use db::Db;
use keys::Key;
use macros::ParseFrames2;

use crate::Frame;

/// <https://redis.io/commands/hdel>
#[derive(Debug, ParseFrames2, Clone)]
pub struct Hdel {
    // todo ref
    pub key: Key,
    pub fields: Vec<Box<[u8]>>,
}

impl From<Hdel> for dict::cmd::kvp::del::Req {
    fn from(old: Hdel) -> Self {
        Self {
            key: old.key,
            fields: old.fields,
        }
    }
}

impl Hdel {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = db.kvp_del(self.into())?;
        Ok(Frame::Integer((response.old_len - response.new_len) as _))
    }
}
