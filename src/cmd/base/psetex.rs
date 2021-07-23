use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db2::Db, slot::data_type::SimpleType, utils::options::NxXx, Frame};
/// https://redis.io/commands/psetex
#[derive(Debug, Clone, ParseFrames)]
pub struct Psetex {
    pub key: SimpleType,
    pub milliseconds: u64,
    pub value: SimpleType,
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
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        db.set(self.into()).await?;
        Ok(Frame::Simple("OK".to_string()))
    }
}
