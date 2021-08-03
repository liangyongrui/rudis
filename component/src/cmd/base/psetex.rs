use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{
    db::Db,
    slot::data_type::DataType,
    utils::{
        now_timestamp_ms,
        options::{ExpiresAt, NxXx},
    },
    Frame,
};
/// https://redis.io/commands/psetex
#[derive(Debug, Clone, ParseFrames)]
pub struct Psetex {
    pub key: Arc<[u8]>,
    pub milliseconds: u64,
    pub value: DataType,
}

impl From<Psetex> for crate::slot::cmd::simple::set::Req {
    fn from(old: Psetex) -> Self {
        Self {
            key: old.key,
            value: old.value,
            expires_at: ExpiresAt::Specific(now_timestamp_ms() + old.milliseconds),
            nx_xx: NxXx::None,
        }
    }
}

impl Psetex {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        db.set(self.into())?;
        Ok(Frame::ok())
    }
}
