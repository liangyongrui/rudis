use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::KeyType, Frame};

/// https://redis.io/commands/incrby
#[derive(Debug, Clone, ParseFrames)]
pub struct Incrby {
    pub key: KeyType,
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
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.incr(self.into()).await?;
        Ok(Frame::Integer(response))
    }
}
