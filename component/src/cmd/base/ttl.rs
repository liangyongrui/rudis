use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::debug;

use crate::{db::Db, Frame};

/// https://redis.io/commands/ttl
#[derive(Debug, Clone, ParseFrames)]
pub struct Ttl {
    pub key: Arc<[u8]>,
}

impl Ttl {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.ttl(dict::cmd::simple::ttl::Req { key: &self.key })?;
        debug!(?res);
        let response = Frame::Integer(match res {
            dict::cmd::simple::ttl::Resp::None => -1,
            dict::cmd::simple::ttl::Resp::NotExist => -2,
            // round
            dict::cmd::simple::ttl::Resp::Ttl(i) => ((i + 500) / 1000) as i64,
        });
        Ok(response)
    }
}
