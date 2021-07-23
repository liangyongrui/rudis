use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{Db, Frame, slot::data_type::SimpleType};

/// https://redis.io/commands/rpushx
#[derive(Debug, Clone, ParseFrames)]
pub struct Rpushx {
    pub key: SimpleType,
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
