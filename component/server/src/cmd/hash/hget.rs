use std::sync::Arc;

use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/hget
#[derive(Debug, ParseFrames)]
pub struct Hget {
    pub key: Arc<[u8]>,
    pub field: Arc<[u8]>,
}

impl<'a> From<&'a Hget> for dict::cmd::kvp::get::Req<'a> {
    fn from(old: &'a Hget) -> Self {
        Self {
            key: &old.key,
            fields: vec![&old.field],
        }
    }
}

impl Hget {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        Ok((&db.kvp_get((&self).into())?[0]).into())
    }
}
