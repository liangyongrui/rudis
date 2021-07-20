use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/hdel
#[derive(Debug, ParseFrames, Clone)]
pub struct Hdel {
    pub key: SimpleType,
    pub fields: Vec<SimpleType>,
}

impl Hdel {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.hdel(&self.key, self.fields) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
