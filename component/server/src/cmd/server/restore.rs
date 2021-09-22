use common::now_timestamp_ms;
use db::Db;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/restore>
#[derive(Debug, ParseFrames)]
pub struct Restore<'a> {
    pub key: &'a [u8],
    pub ttl: u64,
    pub serialized_value: &'a [u8],
    pub replace: bool,
    pub absttl: bool,
}

impl Restore<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let ttl = if self.absttl {
            self.ttl
        } else if self.ttl > 0 {
            now_timestamp_ms() + self.ttl
        } else {
            0
        };
        db.restore(dict::cmd::server::restore::Req {
            key: self.key,
            value: self.serialized_value,
            expires_at: ttl,
            replace: self.replace,
        })?;
        Ok(Frame::ok())
    }
}