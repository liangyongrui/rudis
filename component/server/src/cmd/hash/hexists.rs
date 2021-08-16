use std::sync::Arc;

use db::Db;
use rcc_macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/hexists
#[derive(Debug, ParseFrames)]
pub struct Hexists {
    pub key: Arc<[u8]>,
    pub field: Arc<[u8]>,
}

impl<'a> From<&'a Hexists> for dict::cmd::kvp::exists::Req<'a> {
    fn from(old: &'a Hexists) -> Self {
        Self {
            key: &old.key,
            field: &old.field,
        }
    }
}

impl Hexists {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.kvp_exists((&self).into())?;
        Ok(Frame::Integer(if res { 1 } else { 0 }))
    }
}
