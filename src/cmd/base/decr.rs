use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    Frame,
};

/// https://redis.io/commands/decr
#[derive(Debug, Clone, ParseFrames)]
pub struct Decr {
    pub key: SimpleType,
}

impl Decr {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.incr_by(self.key, -1) {
            Ok(i) => Frame::Integer(i),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
