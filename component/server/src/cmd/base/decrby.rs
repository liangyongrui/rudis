use std::sync::Arc;

use db::Db;
use macros::ParseFrames;

use crate::Frame;
/// https://redis.io/commands/decrby
#[derive(Debug, Clone, ParseFrames)]
pub struct Decrby {
    pub key: Arc<[u8]>,
    pub value: i64,
}

impl From<Decrby> for dict::cmd::simple::incr::Req {
    fn from(old: Decrby) -> Self {
        Self {
            key: old.key,
            value: -old.value,
        }
    }
}

impl Decrby {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = db.incr(self.into())?;
        Ok(Frame::Integer(response))
    }
}
