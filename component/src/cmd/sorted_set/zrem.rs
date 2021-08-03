use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{Db, Frame};

/// https://redis.io/commands/zrem
#[derive(Debug, ParseFrames, Clone)]
pub struct Zrem {
    pub key: Arc<[u8]>,
    pub members: Vec<String>,
}

impl From<Zrem> for crate::slot::cmd::sorted_set::remove::Req {
    fn from(old: Zrem) -> Self {
        Self {
            key: old.key,
            members: old.members,
        }
    }
}

impl Zrem {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let resp = db.sorted_set_remove(self.into())?;
        Ok(Frame::Integer((resp.old_len - resp.new_len) as _))
    }
}
