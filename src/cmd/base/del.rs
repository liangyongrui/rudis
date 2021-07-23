use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::SimpleType, Frame};
/// https://redis.io/commands/del
#[derive(Debug, Clone, ParseFrames)]
pub struct Del {
    pub keys: Vec<SimpleType>,
}

impl Del {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let mut res = 0;
        for cmd in self
            .keys
            .into_iter()
            .map(|key| crate::slot::cmd::simple::del::Req { key })
        {
            if db.del(cmd).await?.is_some() {
                res += 1;
            }
        }
        let response = Frame::Integer(res);
        Ok(response)
    }
}
