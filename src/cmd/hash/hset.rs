use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{
    db::data_type::{SimpleType, SimpleTypePair}, Db, Frame,
};
/// https://redis.io/commands/hset
#[derive(Debug, Clone, ParseFrames)]
pub struct Hset {
    pub key: SimpleType,
    pub pairs: Vec<SimpleTypePair>,
}

impl Hset {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.hset(self.key, self.pairs).await {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
