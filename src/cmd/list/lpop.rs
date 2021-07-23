use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::SimpleType, Frame};

/// https://redis.io/commands/lpop
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpop {
    pub key: SimpleType,
    pub count: Option<i64>,
}

impl From<Lpop> for crate::slot::cmd::deque::pop::Req {
    fn from(old: Lpop) -> Self {
        Self {
            key: old.key,
            count: old.count.filter(|&t| t > 0).unwrap_or(1) as _,
            left: true,
        }
    }
}

impl Lpop {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.deque_pop(self.into()).await?;
        Ok(Frame::Array(res.iter().map(|t| t.into()).collect()))
    }
}
