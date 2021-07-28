use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::SimpleType, Frame};
/// https://redis.io/commands/hincrby
#[derive(Debug, ParseFrames, Clone)]
pub struct Hincrby {
    pub key: Vec<u8>,
    pub field: SimpleType,
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
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let i = db.kvp_incr(self.into()).await?;
        Ok(Frame::Integer(i))
    }
}
