use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, Frame};

/// https://redis.io/commands/expireat
#[derive(Debug, Clone, ParseFrames)]
pub struct Expireat {
    pub key: Arc<[u8]>,
    pub s_timestamp: u64,
}

impl From<Expireat> for crate::slot::cmd::simple::expire::Req {
    fn from(old: Expireat) -> Self {
        Self {
            key: old.key,
            expires_at: old.s_timestamp * 1000,
        }
    }
}

impl Expireat {
    /// Apply the `Set` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.expire(self.into())?;
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
