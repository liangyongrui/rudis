use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    Frame,
};

/// https://redis.io/commands/rpop
#[derive(Debug, Clone, ParseFrames)]
pub struct Rpop {
    pub key: SimpleType,
    pub count: Option<i64>,
}
impl Rpop {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.rpop(&self.key, self.count.unwrap_or(1) as _) {
            Ok(Some(r)) => Frame::Array(r.into_iter().map(|t| t.into()).collect()),
            Ok(None) => Frame::Null,
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
