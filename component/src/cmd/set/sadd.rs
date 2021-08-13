use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{Db, Frame};

/// https://redis.io/commands/sadd
#[derive(Debug, ParseFrames, Clone)]
pub struct Sadd {
    pub key: Arc<[u8]>,
    pub values: Vec<String>,
}

impl From<Sadd> for dict::cmd::set::add::Req {
    fn from(old: Sadd) -> Self {
        Self {
            key: old.key,
            members: old.values,
        }
    }
}

impl Sadd {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.set_add(self.into())?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
