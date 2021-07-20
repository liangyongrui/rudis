use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/hexists
#[derive(Debug, ParseFrames)]
pub struct Hexists {
    pub key: SimpleType,
    pub field: SimpleType,
}

impl Hexists {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.hexists(&self.key, self.field) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
