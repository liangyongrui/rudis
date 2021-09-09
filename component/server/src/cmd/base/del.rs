use connection::Connection;
use db::Db;
use keys::Key;
use macros::ParseFrames2;
use tracing::error;

use crate::Frame;
/// <https://redis.io/commands/del>
#[derive(Debug, Clone, ParseFrames2)]
pub struct Del {
    // todo ref
    pub keys: Vec<Key>,
}

impl Del {
    #[tracing::instrument(skip(self, connection, db), level = "debug")]
    pub async fn apply<'a>(
        self,
        connection: &'a mut Connection,
        db: &Db,
    ) -> common::Result<Frame<'a>> {
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
        let response = Frame::Integer(res);
        if let Err(e) = connection.write_frame(&response).await {
            error!("connection error: {:?}", e);
        }
        Ok(Frame::NoRes)
    }
}
