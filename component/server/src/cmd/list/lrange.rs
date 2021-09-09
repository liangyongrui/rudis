use db::Db;
use macros::ParseFrames2;

use crate::{frame_parse::data_type_to_frame, Frame};

/// <https://redis.io/commands/lrange>
#[derive(Debug, ParseFrames2)]
pub struct Lrange<'a> {
    pub key: &'a [u8],
    pub start: i64,
    pub stop: i64,
}

impl Lrange<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = db.deque_range(dict::cmd::deque::range::Req {
            key: self.key,
            start: self.start,
            stop: self.stop,
        })?;
        Ok(Frame::Array(
            response.into_iter().map(data_type_to_frame).collect(),
        ))
    }
}
