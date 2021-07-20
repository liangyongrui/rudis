use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/hsetnx
#[derive(Debug, ParseFrames, Clone)]
pub struct Hsetnx {
    pub key: SimpleType,
    pub field: SimpleType,
    pub value: SimpleType,
}

impl Hsetnx {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.hsetnx(self.key, self.field, self.value).await {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
