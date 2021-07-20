use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};
/// https://redis.io/commands/zrank
#[derive(Debug, ParseFrames)]
pub struct Zrank {
    pub key: SimpleType,
    pub member: SimpleType,
}

impl Zrank {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.zrank(&self.key, &self.member, false) {
            Ok(None) => Frame::Null,
            Ok(Some(v)) => Frame::Integer(v as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
