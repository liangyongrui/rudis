use std::{sync::Arc, vec};

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};
/// https://redis.io/commands/hmget
#[derive(Debug, ParseFrames)]
pub struct Hmget {
    pub key: Arc<[u8]>,
    pub fields: Vec<String>,
}

impl Hmget {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let v = db.kvp_get(dict::cmd::kvp::get::Req {
            key: &self.key,
            fields: self
                .fields
                .iter()
                .map(std::string::String::as_str)
                .collect(),
        })?;
        let res = v.into_iter().map(|i| i.into()).collect();
        Ok(Frame::Array(res))
    }
}
