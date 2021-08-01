use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::Db,
    slot::data_type::DataType,
    utils::{
        now_timestamp_ms,
        options::{ExpiresAt, NxXx},
    },
    Frame,
};
/// https://redis.io/commands/setex
#[derive(Debug, Clone, ParseFrames)]
pub struct Setex {
    pub key: Arc<[u8]>,
    pub seconds: u64,
    pub value: DataType,
}

impl From<Setex> for crate::slot::cmd::simple::set::Req {
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
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        db.set(self.into())?;
        Ok(Frame::ok())
    }
}
