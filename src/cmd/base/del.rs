use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};
/// https://redis.io/commands/del
#[derive(Debug, Clone, ParseFrames)]
pub struct Del {
    pub keys: Vec<SimpleType>,
}

impl Del {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = Frame::Integer(db.del(self.keys) as i64);
        Ok(response)
    }
}
