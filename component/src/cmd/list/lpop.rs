use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/lpop
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpop {
    pub key: Arc<[u8]>,
    pub count: Option<i64>,
}

impl From<Lpop> for dict::cmd::deque::pop::Req {
    fn from(old: Lpop) -> Self {
        Self {
            key: old.key,
            count: old.count.filter(|&t| t > 0).unwrap_or(1) as _,
            left: true,
        }
    }
}

impl Lpop {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.deque_pop(self.into())?;
        Ok(Frame::Array(res.into_iter().map(|t| t.into()).collect()))
    }
}
