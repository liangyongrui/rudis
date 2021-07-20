use rcc_macros::ParseFrames;
use tracing::{instrument};

use crate::{db::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/zremrangebyrank
#[derive(Debug, Clone, ParseFrames)]
pub struct Zremrangebyrank {
    pub key: SimpleType,
    pub range: (i64, i64),
}

impl Zremrangebyrank {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.zremrange_by_rank(&self.key, self.range) {
            Ok(v) => Frame::Integer(v as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
