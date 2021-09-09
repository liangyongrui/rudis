use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::{frame_parse::data_type_to_frame, Frame};

/// <https://redis.io/commands/lpop>
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpop {
    pub key: Key,
    #[default(1)]
    pub count: usize,
}

impl Lpop {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.deque_pop(dict::cmd::deque::pop::Req {
            key: self.key,
            count: self.count,
            left: true,
        })?;
        Ok(Frame::Array(
            res.into_iter().map(data_type_to_frame).collect(),
        ))
    }
}
