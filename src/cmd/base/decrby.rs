use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    Frame,
};

/// https://redis.io/commands/decrby
#[derive(Debug, Clone, ParseFrames)]
pub struct Decrby {
    pub key: SimpleType,
    pub value: i64,
}
impl Decrby {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.incr_by(self.key, -self.value) {
            Ok(i) => Frame::Integer(i),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
