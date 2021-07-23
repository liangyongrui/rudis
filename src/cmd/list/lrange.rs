use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db2::Db, slot::data_type::SimpleType, Frame};

/// https://redis.io/commands/lrange
#[derive(Debug, ParseFrames)]
pub struct Lrange {
    pub key: SimpleType,
    pub start: i64,
    pub stop: i64,
}

impl From<Lrange> for crate::slot::cmd::deque::range::Req<'_> {
    fn from(old: Lrange) -> Self {
        Self {
            key: &old.key,
            start: old.start,
            stop: old.stop,
        }
    }
}
impl Lrange {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_range(self.into())?;
        Ok(Frame::Array(response.iter().map(|t| t.into()).collect()))
    }
}
