use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/lrange
#[derive(Debug, ParseFrames)]
pub struct Lrange {
    pub key: Arc<[u8]>,
    pub start: i64,
    pub stop: i64,
}

impl<'a> From<&'a Lrange> for dict::cmd::deque::range::Req<'a> {
    fn from(old: &'a Lrange) -> Self {
        Self {
            key: &old.key,
            start: old.start,
            stop: old.stop,
        }
    }
}
impl Lrange {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_range((&self).into())?;
        Ok(Frame::Array(
            response.into_iter().map(|t| t.into()).collect(),
        ))
    }
}
