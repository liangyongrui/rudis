use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/smembers
#[derive(Debug, ParseFrames)]
pub struct Smembers {
    pub key: SimpleType,
}

impl Smembers {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.smembers(&self.key) {
            Ok(i) => Frame::Array(i.iter().map(|t| t.clone().into()).collect()),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
