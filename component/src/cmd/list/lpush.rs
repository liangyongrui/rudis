use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{db::Db, slot::data_type::DataType, Frame};

/// https://redis.io/commands/lpush
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpush {
    pub key: Arc<[u8]>,
    pub values: Vec<DataType>,
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
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_push(self.into())?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
