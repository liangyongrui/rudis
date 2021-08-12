use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/hget
#[derive(Debug, ParseFrames)]
pub struct Hget {
    pub key: Arc<[u8]>,
    pub field: String,
}

impl<'a> From<&'a Hget> for crate::slot::cmd::kvp::get::Req<'a> {
    fn from(old: &'a Hget) -> Self {
        Self {
            key: &old.key,
            fields: vec![&old.field],
        }
    }
}

impl Hget {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        Ok((&db.kvp_get((&self).into())?[0]).into())
    }
}
