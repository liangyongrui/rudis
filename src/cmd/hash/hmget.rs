use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{db::data_type::SimpleType, Db, Frame};
/// https://redis.io/commands/hmget
#[derive(Debug, ParseFrames)]
pub struct Hmget {
    pub key: SimpleType,
    pub fields: Vec<SimpleType>,
}

impl Hmget {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.hmget(&self.key, self.fields) {
            Ok(v) => Frame::Array(
                v.into_iter()
                    .map(|x| x.map(|y| y.into()).unwrap_or(Frame::Null))
                    .collect(),
            ),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
