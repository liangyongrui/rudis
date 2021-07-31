use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::instrument;

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
            field: &old.field,
        }
    }
}

impl Hget {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        Ok((&db.kvp_get((&self).into())?).into())
    }
}
