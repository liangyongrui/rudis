use std::ops::Bound;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{slot::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/zremrangebyrank
#[derive(Debug, Clone, ParseFrames)]
pub struct Zremrangebylex {
    pub key: SimpleType,
    pub min: String,
    pub max: String,
}

impl From<Zremrangebylex> for crate::slot::cmd::sorted_set::remove_by_lex_range::Req {
    fn from(old: Zremrangebylex) -> Self {
        let min = old.min;
        let max = old.max;
        let range = {
            let min = if min == "-" {
                Bound::Unbounded
            } else if let Some(s) = min.strip_prefix('(') {
                Bound::Excluded(SimpleType::String(s.into()))
            } else {
                Bound::Included(SimpleType::String(min[1..].into()))
            };
            let max = if max == "+" {
                Bound::Unbounded
            } else if let Some(s) = max.strip_prefix('(') {
                Bound::Excluded(SimpleType::String(s.into()))
            } else {
                Bound::Included(SimpleType::String(max[1..].into()))
            };
            (min, max)
        };
        Self {
            key: old.key,
            rev: false,
            range,
        }
    }
}

impl Zremrangebylex {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.sorted_set_remove_by_lex_range(self.into()).await?;
        Ok(Frame::Integer(res.len() as _))
    }
}
