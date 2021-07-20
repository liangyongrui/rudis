use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{
    db::{data_type::SimpleType, Db}, Frame,
};

/// https://redis.io/commands/llen
#[derive(Debug, ParseFrames)]
pub struct Llen {
    pub key: SimpleType,
}
impl Llen {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.llen(&self.key) {
            Ok(r) => Frame::Integer(r as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
