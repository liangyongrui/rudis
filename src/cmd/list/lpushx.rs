use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/lpushx
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpushx {
    pub key: SimpleType,
    pub values: Vec<SimpleType>,
}

impl Lpushx {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.lpushx(&self.key, self.values) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
