use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/exists>
#[derive(Debug, ParseFrames)]
pub struct Exists {
    pub keys: Vec<Key>,
}

impl Exists {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let mut res = 0;
        for cmd in self
            .keys
            .iter()
            .map(|key| dict::cmd::simple::exists::Req { key })
        {
            if db.exists(cmd)? {
                res += 1;
            }
        }
        let response = Frame::Integer(res);
        Ok(response)
    }
}
