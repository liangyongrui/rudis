use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db2::Db, slot::data_type::SimpleType, Frame};

/// https://redis.io/commands/exists
#[derive(Debug, ParseFrames)]
pub struct Exists {
    pub keys: Vec<SimpleType>,
}

impl Exists {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
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
