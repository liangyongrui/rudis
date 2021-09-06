use db::Db;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/ttl>
#[derive(Debug, Clone, ParseFrames)]
pub struct Ttl {
    pub key: Key,
}

impl Ttl {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.ttl(dict::cmd::simple::ttl::Req { key: &self.key })?;
        let response = Frame::Integer(match res {
            dict::cmd::simple::ttl::Resp::None => -1,
            dict::cmd::simple::ttl::Resp::NotExist => -2,
            // round
            #[allow(clippy::cast_possible_wrap)]
            dict::cmd::simple::ttl::Resp::Ttl(i) => ((i + 500) / 1000) as i64,
        });
        Ok(response)
    }
}
