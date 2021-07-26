use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::KeyType, Frame};

/// https://redis.io/commands/rpop
#[derive(Debug, Clone, ParseFrames)]
pub struct Rpop {
    pub key: KeyType,
    pub count: Option<i64>,
}

impl From<Rpop> for crate::slot::cmd::deque::pop::Req {
    fn from(old: Rpop) -> Self {
        Self {
            key: old.key,
            count: old.count.filter(|&t| t > 0).unwrap_or(1) as _,
            left: false,
        }
    }
}
impl Rpop {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.deque_pop(self.into()).await?;
        Ok(Frame::Array(res.iter().map(|t| t.into()).collect()))
    }
}
