use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/hdel
#[derive(Debug, ParseFrames, Clone)]
pub struct Hdel {
    pub key: Arc<[u8]>,
    pub fields: Vec<String>,
}

impl From<Hdel> for dict::cmd::kvp::del::Req {
    fn from(old: Hdel) -> Self {
        Self {
            key: old.key,
            fields: old.fields,
        }
    }
}

impl Hdel {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.kvp_del(self.into())?;
        Ok(Frame::Integer((response.old_len - response.new_len) as _))
    }
}
