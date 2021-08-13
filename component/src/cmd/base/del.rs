use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};
/// https://redis.io/commands/del
#[derive(Debug, Clone, ParseFrames)]
pub struct Del {
    pub keys: Vec<Arc<[u8]>>,
}

impl Del {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let mut res = 0;
        for cmd in self
            .keys
            .into_iter()
            .map(|key| dict::cmd::simple::del::Req { key })
        {
            if db.del(cmd)?.is_some() {
                res += 1;
            }
        }
        let response = Frame::Integer(res);
        Ok(response)
    }
}
