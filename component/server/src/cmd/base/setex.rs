use common::{
    now_timestamp_ms,
    options::{ExpiresAt, NxXx},
};
use db::Db;
use dict::data_type::DataType;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/setex>
#[derive(Debug, ParseFrames)]
pub struct Setex {
    pub key: Key,
    pub seconds: u64,
    pub value: DataType,
}

impl From<Setex> for dict::cmd::simple::set::Req {
    #[inline]
    fn from(old: Setex) -> Self {
        Self {
            key: old.key,
            value: old.value,
            expires_at: ExpiresAt::Specific(now_timestamp_ms() + old.seconds * 1000),
            nx_xx: NxXx::None,
        }
    }
}

impl Setex {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        db.set(self.into())?;
        Ok(Frame::ok())
    }
}
