use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};
/// https://redis.io/commands/smismember
#[derive(Debug, ParseFrames)]
pub struct Smismember {
    pub key: SimpleType,
    pub values: Vec<SimpleType>,
}

impl Smismember {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.smismember(&self.key, self.values) {
            Ok(i) => Frame::Array(
                i.into_iter()
                    .map(|t| if t { 1 } else { 0 })
                    .map(Frame::Integer)
                    .collect(),
            ),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
