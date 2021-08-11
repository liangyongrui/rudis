use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/pttl
#[derive(Debug, Clone, ParseFrames)]
pub struct Pttl {
    pub key: Arc<[u8]>,
}

impl Pttl {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.ttl(crate::slot::cmd::simple::ttl::Req { key: &self.key })?;
        let response = Frame::Integer(match res {
            crate::slot::cmd::simple::ttl::Resp::None => -1,
            crate::slot::cmd::simple::ttl::Resp::NotExist => -2,
            crate::slot::cmd::simple::ttl::Resp::Ttl(i) => i as i64,
        });
        Ok(response)
    }
}
