use db::Db;
use keys::Key;
use macros::ParseFrames;
use tracing::debug;

use crate::Frame;
/// <https://redis.io/commands/del>
#[derive(Debug, ParseFrames)]
pub struct Del {
    pub keys: Vec<Key>,
}

impl Del {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply<'a>(self, db: &Db) -> common::Result<Frame<'a>> {
        let mut res = 0;
        let mut delay = Vec::with_capacity(self.keys.len());
        for cmd in self
            .keys
            .into_iter()
            .map(|key| dict::cmd::simple::del::Req { key })
        {
            let r = db.del(cmd)?;
            if r.is_some() {
                res += 1;
            }
            delay.push(r);
        }
        // async drop
        tokio::spawn(async { delay });
        debug!("{}", res);
        Ok(Frame::Integer(res))
    }
}
