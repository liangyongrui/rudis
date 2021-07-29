use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, Frame};

/// https://redis.io/commands/decrby
#[derive(Debug, Clone, ParseFrames)]
pub struct Decrby {
    pub key: Vec<u8>,
    pub value: i64,
}

impl From<Decrby> for crate::slot::cmd::simple::incr::Req {
    fn from(old: Decrby) -> Self {
        Self {
            key: old.key,
            value: -old.value,
        }
    }
}

impl Decrby {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.incr(self.into())?;
        Ok(Frame::Integer(response))
    }
}
