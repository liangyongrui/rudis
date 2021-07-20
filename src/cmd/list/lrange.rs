use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/lrange
#[derive(Debug, ParseFrames)]
pub struct Lrange {
    pub key: SimpleType,
    pub start: i64,
    pub stop: i64,
}

impl Lrange {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.lrange(&self.key, self.start, self.stop) {
            Ok(r) => Frame::Array(r.into_iter().map(|t| t.into()).collect()),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
