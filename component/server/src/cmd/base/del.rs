use std::sync::Arc;

use connection::Connection;
use db::Db;
use macros::ParseFrames;
use tracing::error;

use crate::Frame;
/// https://redis.io/commands/del
#[derive(Debug, Clone, ParseFrames)]
pub struct Del {
    pub keys: Vec<Arc<[u8]>>,
}

impl Del {
    #[tracing::instrument(skip(self, connection, db), level = "debug")]
    pub async fn apply(self, connection: &mut Connection, db: &Db) -> common::Result<Frame> {
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
            error!("{:?}", e);
        }
        Ok(Frame::NoRes)
    }
}
