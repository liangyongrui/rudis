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
        if let Some(all) = db.kvp_get_all(crate::slot::cmd::kvp::get_all::Req { key: &self.key })? {
            let res = self
                .fields
                .iter()
                .map(|f| all.get(f).map(|t| t.into()).unwrap_or(Frame::Null))
                .collect();
            Ok(Frame::Array(res))
        } else {
            Ok(Frame::Array(vec![Frame::Null; self.fields.len()]))
        }
    }
}
