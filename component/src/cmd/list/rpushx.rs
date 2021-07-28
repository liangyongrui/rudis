use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{slot::data_type::SimpleType, Db, Frame};

/// https://redis.io/commands/rpushx
#[derive(Debug, Clone, ParseFrames)]
pub struct Rpushx {
    pub key: Vec<u8>,
    pub values: Vec<SimpleType>,
}

impl From<Rpushx> for crate::slot::cmd::deque::push::Req {
    fn from(old: Rpushx) -> Self {
        Self {
            key: old.key,
            left: false,
            elements: old.values,
            nx_xx: crate::utils::options::NxXx::Xx,
        }
    }
}

impl Rpushx {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_push(self.into()).await?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
