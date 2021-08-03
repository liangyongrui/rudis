use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{Db, Frame};

/// https://redis.io/commands/sismember
#[derive(Debug, ParseFrames)]
pub struct Sismember {
    pub key: Arc<[u8]>,
    pub value: String,
}

impl<'a> From<&'a Sismember> for crate::slot::cmd::set::exists::Req<'a> {
    fn from(old: &'a Sismember) -> Self {
        Self {
            key: &old.key,
            field: &old.value,
        }
    }
}

impl Sismember {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.set_exists((&self).into())?;
        Ok(Frame::Integer(if res { 1 } else { 0 }))
    }
}
