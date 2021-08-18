use std::sync::Arc;

use db::Db;
use rcc_macros::ParseFrames;

use crate::Frame;

/// https://redis.io/commands/rpop
#[derive(Debug, Clone, ParseFrames)]
pub struct Rpop {
    pub key: Arc<[u8]>,
    pub count: Option<i64>,
}

impl From<Rpop> for dict::cmd::deque::pop::Req {
    fn from(old: Rpop) -> Self {
        Self {
            key: old.key,
            count: old.count.filter(|&t| t > 0).unwrap_or(1) as _,
            left: false,
        }
    }
}
impl Rpop {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.deque_pop(self.into())?;
        Ok(Frame::Array(res.into_iter().map(|t| t.into()).collect()))
    }
}