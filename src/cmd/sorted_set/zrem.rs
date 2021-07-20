use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/zrem
#[derive(Debug, ParseFrames, Clone)]
pub struct Zrem {
    pub key: SimpleType,
    pub members: Vec<SimpleType>,
}

impl Zrem {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.zrem(&self.key, self.members) {
            Ok(v) => Frame::Integer(v as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
