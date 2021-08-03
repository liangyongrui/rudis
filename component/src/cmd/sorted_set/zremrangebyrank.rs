use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{Db, Frame};

/// https://redis.io/commands/zremrangebyrank
#[derive(Debug, Clone, ParseFrames)]
pub struct Zremrangebyrank {
    pub key: Arc<[u8]>,
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
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.sorted_set_remove_by_rank_range(self.into())?;
        Ok(Frame::Integer(res.len() as _))
    }
}
