use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/rpush
#[derive(Debug, Clone, ParseFrames)]
pub struct Rpush {
    pub key: SimpleType,
    pub values: Vec<SimpleType>,
}

impl Rpush {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.rpush(self.key, self.values).await {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
