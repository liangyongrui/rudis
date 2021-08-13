use std::sync::Arc;

use common::options::NxXx;
use dict::data_type::DataType;
use rcc_macros::ParseFrames;

use crate::{db::Db, Frame};

/// https://redis.io/commands/rpush
#[derive(Debug, Clone, ParseFrames)]
pub struct Rpush {
    pub key: Arc<[u8]>,
    pub values: Vec<DataType>,
}

impl From<Rpush> for dict::cmd::deque::push::Req {
    fn from(old: Rpush) -> Self {
        Self {
            key: old.key,
            left: false,
            elements: old.values,
            nx_xx: NxXx::None,
        }
    }
}

impl Rpush {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_push(self.into())?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
