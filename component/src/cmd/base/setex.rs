use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::SimpleType, utils::options::NxXx, Frame};
/// https://redis.io/commands/setex
#[derive(Debug, Clone, ParseFrames)]
pub struct Setex {
    pub key: Vec<u8>,
    pub seconds: u64,
    pub value: SimpleType,
}

impl From<Setex> for crate::slot::cmd::simple::set::Req {
    fn from(old: Setex) -> Self {
        Self {
            key: old.key,
            value: old.value,
            expires_at: Utc::now()
                .checked_add_signed(Duration::seconds(old.seconds as _))
                .into(),
            nx_xx: NxXx::None,
        }
    }
}

impl Setex {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        db.set(self.into())?;
        Ok(Frame::Simple("OK".to_string()))
    }
}
