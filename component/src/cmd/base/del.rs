use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, Frame};
/// https://redis.io/commands/del
#[derive(Debug, Clone, ParseFrames)]
pub struct Del {
    pub keys: Vec<Vec<u8>>,
}

impl Del {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let mut res = 0;
        for cmd in self
            .keys
            .into_iter()
            .map(|key| crate::slot::cmd::simple::del::Req { key })
        {
            if db.del(cmd)?.is_some() {
                res += 1;
            }
        }
        let response = Frame::Integer(res);
        Ok(response)
    }
}
