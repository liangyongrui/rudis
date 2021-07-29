use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::Db, slot::data_type::SimpleType, Frame};

/// https://redis.io/commands/lpushx
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpushx {
    pub key: Vec<u8>,
    pub values: Vec<SimpleType>,
}

impl From<Lpushx> for crate::slot::cmd::deque::push::Req {
    fn from(old: Lpushx) -> Self {
        Self {
            key: old.key,
            left: true,
            elements: old.values,
            nx_xx: crate::utils::options::NxXx::Xx,
        }
    }
}

impl Lpushx {
    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_push(self.into())?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
