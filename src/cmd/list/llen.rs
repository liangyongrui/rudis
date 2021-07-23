use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db2::Db, slot::data_type::SimpleType, Frame};

/// https://redis.io/commands/llen
#[derive(Debug, ParseFrames)]
pub struct Llen {
    pub key: SimpleType,
}

impl From<Llen> for crate::slot::cmd::deque::len::Req<'_> {
    fn from(old: Llen) -> Self {
        Self { key: &old.key }
    }
}

impl Llen {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let i = db.deque_len(self.into())?;
        Ok(Frame::Integer(i as _))
    }
}
