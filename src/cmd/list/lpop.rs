use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    Frame,
};

/// https://redis.io/commands/lpop
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpop {
    pub key: SimpleType,
    pub count: Option<i64>,
}
impl Lpop {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.lpop(&self.key, self.count.unwrap_or(1) as _) {
            Ok(Some(r)) => Frame::Array(r.into_iter().map(|t| t.into()).collect()),
            Ok(None) => Frame::Null,
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
