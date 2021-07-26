use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::KeyType, Frame};

/// https://redis.io/commands/llen
#[derive(Debug, ParseFrames)]
pub struct Llen {
    pub key: KeyType,
}

impl<'a> From<&'a Llen> for crate::slot::cmd::deque::len::Req<'a> {
    fn from(old: &'a Llen) -> Self {
        Self { key: &old.key }
    }
}

impl Llen {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let i = db.deque_len((&self).into())?;
        Ok(Frame::Integer(i as _))
    }
}
