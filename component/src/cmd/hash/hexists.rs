use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, Frame};

/// https://redis.io/commands/hexists
#[derive(Debug, ParseFrames)]
pub struct Hexists {
    pub key: Arc<[u8]>,
    pub field: String,
}

impl<'a> From<&'a Hexists> for crate::slot::cmd::kvp::exists::Req<'a> {
    fn from(old: &'a Hexists) -> Self {
        Self {
            key: &old.key,
            field: &old.field,
        }
    }
}

impl Hexists {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.kvp_exists((&self).into())?;
        Ok(Frame::Integer(if res { 1 } else { 0 }))
    }
}
