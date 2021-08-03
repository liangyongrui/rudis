use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};
/// https://redis.io/commands/hincrby
#[derive(Debug, ParseFrames, Clone)]
pub struct Hincrby {
    pub key: Arc<[u8]>,
    pub field: String,
    pub value: i64,
}

impl From<Hincrby> for crate::slot::cmd::kvp::incr::Req {
    fn from(old: Hincrby) -> Self {
        Self {
            key: old.key,
            field: old.field,
            value: old.value,
        }
    }
}

impl Hincrby {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let i = db.kvp_incr(self.into())?;
        Ok(Frame::Integer(i))
    }
}
