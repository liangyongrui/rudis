use std::sync::Arc;

use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::DataType, utils::options::NxXx, Frame};
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
            expires_at: Utc::now()
                .checked_add_signed(Duration::milliseconds(old.milliseconds as _))
                .into(),
            nx_xx: NxXx::None,
        }
    }
}

impl Psetex {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        db.set(self.into())?;
        Ok(Frame::Simple("OK".to_string()))
    }
}
