use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db}, Frame,
};

/// https://redis.io/commands/pexpire
#[derive(Debug, Clone, ParseFrames)]
pub struct Pexpire {
    pub key: SimpleType,
    pub milliseconds: u64,
}
impl Pexpire {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = if let Some(ea) =
            Utc::now().checked_add_signed(Duration::milliseconds(self.milliseconds as _))
        {
            db.expires_at(&self.key, ea).await
        } else {
            false
        };
        // Create a success response and write it to `dst`.
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
