use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/llen
#[derive(Debug, ParseFrames)]
pub struct Llen {
    pub key: Arc<[u8]>,
}

impl<'a> From<&'a Llen> for crate::slot::cmd::deque::len::Req<'a> {
    fn from(old: &'a Llen) -> Self {
        Self { key: &old.key }
    }
}

impl Llen {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let i = db.deque_len((&self).into())?;
        Ok(Frame::Integer(i as _))
    }
}
