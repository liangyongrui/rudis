use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/sadd
#[derive(Debug, ParseFrames, Clone)]
pub struct Sadd {
    pub key: SimpleType,
    pub values: Vec<SimpleType>,
}

impl Sadd {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.sadd(self.key, self.values).await {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
