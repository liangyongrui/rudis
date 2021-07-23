use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db2::Db, slot::data_type::SimpleType, Frame};

/// https://redis.io/commands/pexpire
#[derive(Debug, Clone, ParseFrames)]
pub struct Pexpire {
    pub key: SimpleType,
    pub milliseconds: u64,
}

impl From<Pexpire> for crate::slot::cmd::simple::expire::Req {
    fn from(old: Pexpire) -> Self {
        Self {
            key: old.key,
            expire_at: Utc::now().checked_add_signed(Duration::milliseconds(old.milliseconds as _)),
        }
    }
}

impl Pexpire {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.expire(self.into()).await?;
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
