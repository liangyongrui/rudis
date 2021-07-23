use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db2::Db, slot::data_type::SimpleType, Frame};

/// https://redis.io/commands/lpush
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpush {
    pub key: SimpleType,
    pub values: Vec<SimpleType>,
}

impl From<Lpush> for crate::slot::cmd::deque::push::Req {
    fn from(old: Lpush) -> Self {
        Self {
            key: old.key,
            left: true,
            elements: old.values,
            nx_xx: crate::utils::options::NxXx::None,
        }
    }
}

impl Lpush {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_push(self.into()).await?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
