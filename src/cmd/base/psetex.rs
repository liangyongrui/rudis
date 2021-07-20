use chrono::{Duration, Utc};
use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{
    db::{data_type::SimpleType, Db},
    utils::options::NxXx,
    Frame,
};
/// https://redis.io/commands/psetex
#[derive(Debug, Clone, ParseFrames)]
pub struct Psetex {
    pub key: SimpleType,
    pub milliseconds: u64,
    pub value: SimpleType,
}
impl Psetex {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = if let Err(e) = db
            .set(
                self.key,
                self.value,
                NxXx::None,
                Utc::now().checked_add_signed(Duration::milliseconds(self.milliseconds as i64)),
                false,
            )
            .await
        {
            Frame::Error(e)
        } else {
            Frame::Simple("OK".to_string())
        };
        // Create a success response and write it to `dst`.
        Ok(response)
    }
}
