use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/hget
#[derive(Debug, ParseFrames)]
pub struct Hget {
    pub key: SimpleType,
    pub field: SimpleType,
}

impl Hget {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.hget(&self.key, self.field) {
            Ok(Some(v)) => v.into(),
            Ok(None) => Frame::Null,
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
