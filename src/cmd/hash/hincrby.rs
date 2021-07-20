use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};
/// https://redis.io/commands/hincrby
#[derive(Debug, ParseFrames, Clone)]
pub struct Hincrby {
    pub key: SimpleType,
    pub field: SimpleType,
    pub value: i64,
}

impl Hincrby {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.hincrby(self.key, self.field, self.value).await {
            Ok(i) => Frame::Integer(i),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
