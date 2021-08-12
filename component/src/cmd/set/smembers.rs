use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{Db, Frame};

/// https://redis.io/commands/smembers
#[derive(Debug, ParseFrames)]
pub struct Smembers {
    pub key: Arc<[u8]>,
}

impl<'a> From<&'a Smembers> for crate::slot::cmd::set::get_all::Req<'a> {
    fn from(old: &'a Smembers) -> Self {
        Self { key: &old.key }
    }
}

impl Smembers {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.set_get_all((&self).into())?;
        Ok(Frame::Array(
            res.iter().map(|t| Frame::Simple((&t[..]).into())).collect(),
        ))
    }
}
