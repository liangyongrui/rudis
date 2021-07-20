use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/sismember
#[derive(Debug, ParseFrames)]
pub struct Sismember {
    pub key: SimpleType,
    pub value: SimpleType,
}

impl Sismember {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.sismember(&self.key, self.value) {
            Ok(i) => Frame::Integer(if i { 1 } else { 0 }),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
