use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/debug-object>
#[derive(Debug, ParseFrames)]
pub struct Debug<'a> {
    pub sub_cmd: &'a [u8],
    pub payload: Vec<&'a [u8]>,
}

impl Debug<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        if self.sub_cmd == b"object"
            && !db.exists(dict::cmd::simple::exists::Req {
                key: self.payload[0],
            })?
        {
            return Err("ERR no such key".into());
        }
        Ok(Frame::ok())
    }
}
