use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::{frame_parse::data_type_to_frame, Frame};

/// <https://redis.io/commands/rpop>
#[derive(Debug, ParseFrames)]
pub struct Rpop {
    pub key: Key,
    #[default(1)]
    pub count: usize,
}

impl Rpop {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.deque_pop(dict::cmd::deque::pop::Req {
            key: self.key,
            count: self.count,
            left: false,
        })?;
        Ok(Frame::Array(
            res.into_iter().map(data_type_to_frame).collect(),
        ))
    }
}
