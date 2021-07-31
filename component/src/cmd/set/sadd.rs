use std::sync::Arc;

use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{Db, Frame};

/// https://redis.io/commands/sadd
#[derive(Debug, ParseFrames, Clone)]
pub struct Sadd {
    pub key: Arc<[u8]>,
    pub values: Vec<String>,
}

impl From<Sadd> for crate::slot::cmd::set::add::Req {
    fn from(old: Sadd) -> Self {
        Self {
            key: old.key,
            members: old.values,
        }
    }
}

impl Sadd {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.set_add(self.into())?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
