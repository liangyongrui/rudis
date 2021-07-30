use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, Frame};

/// https://redis.io/commands/incrby
#[derive(Debug, Clone, ParseFrames)]
pub struct Incrby {
    pub key: Arc<[u8]>,
    pub value: i64,
}

impl From<Incrby> for crate::slot::cmd::simple::incr::Req {
    fn from(old: Incrby) -> Self {
        Self {
            key: old.key,
            value: old.value,
        }
    }
}
impl Incrby {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.incr(self.into())?;
        Ok(Frame::Integer(response))
    }
}
