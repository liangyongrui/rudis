use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/exists
#[derive(Debug, ParseFrames)]
pub struct Exists {
    pub keys: Vec<SimpleType>,
}

impl Exists {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = Frame::Integer(db.exists(self.keys) as i64);
        Ok(response)
    }
}
