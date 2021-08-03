use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/exists
#[derive(Debug, ParseFrames)]
pub struct Exists {
    pub keys: Vec<Arc<[u8]>>,
}

impl Exists {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let mut res = 0;
        for cmd in self
            .keys
            .iter()
            .map(|key| crate::slot::cmd::simple::exists::Req { key })
        {
            if db.exists(cmd)? {
                res += 1;
            }
        }
        let response = Frame::Integer(res);
        Ok(response)
    }
}
