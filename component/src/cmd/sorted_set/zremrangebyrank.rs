use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{slot::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/zremrangebyrank
#[derive(Debug, Clone, ParseFrames)]
pub struct Zremrangebyrank {
    pub key: SimpleType,
    pub range: (i64, i64),
}

impl From<Zremrangebyrank> for crate::slot::cmd::sorted_set::remove_by_rank_range::Req {
    fn from(old: Zremrangebyrank) -> Self {
        Self {
            key: old.key,
            start: old.range.0,
            stop: old.range.1,
            rev: false,
        }
    }
}

impl Zremrangebyrank {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.sorted_set_remove_by_rank_range(self.into()).await?;
        Ok(Frame::Integer(res.len() as _))
    }
}