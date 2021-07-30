use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, Frame};

/// https://redis.io/commands/lrange
#[derive(Debug, ParseFrames)]
pub struct Lrange {
    pub key: Arc<[u8]>,
    pub start: i64,
    pub stop: i64,
}

impl<'a> From<&'a Lrange> for crate::slot::cmd::deque::range::Req<'a> {
    fn from(old: &'a Lrange) -> Self {
        Self {
            key: &old.key,
            start: old.start,
            stop: old.stop,
        }
    }
}
impl Lrange {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_range((&self).into())?;
        Ok(Frame::Array(response.iter().map(|t| t.into()).collect()))
    }
}
