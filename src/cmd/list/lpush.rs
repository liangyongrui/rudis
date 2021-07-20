use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/lpush
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpush {
    pub key: SimpleType,
    pub values: Vec<SimpleType>,
}

impl Lpush {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.lpush(self.key, self.values).await {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
