use db::Db;
use macros::ParseFrames2;

use crate::Frame;

/// <https://redis.io/commands/exists>
#[derive(Debug, ParseFrames2)]
pub struct Exists<'a> {
    pub keys: Vec<&'a [u8]>,
}

impl Exists<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let mut res = 0;
        for cmd in self
            .keys
            .into_iter()
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
