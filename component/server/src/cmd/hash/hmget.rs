use std::{sync::Arc, vec};

use db::Db;
use macros::ParseFrames;
use nom::AsBytes;

use crate::Frame;

/// https://redis.io/commands/hmget
#[derive(Debug, ParseFrames)]
pub struct Hmget {
    pub key: Arc<[u8]>,
    pub fields: Vec<Arc<[u8]>>,
}

impl Hmget {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let v = db.kvp_get(dict::cmd::kvp::get::Req {
            key: &self.key,
            fields: self.fields.iter().map(|t| t.as_bytes()).collect(),
        })?;
        let res = v.into_iter().map(|i| i.into()).collect();
        Ok(Frame::Array(res))
    }
}
