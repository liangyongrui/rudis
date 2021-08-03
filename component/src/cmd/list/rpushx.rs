use std::sync::Arc;

use rcc_macros::ParseFrames;

use crate::{slot::data_type::DataType, Db, Frame};

/// https://redis.io/commands/rpushx
#[derive(Debug, Clone, ParseFrames)]
pub struct Rpushx {
    pub key: Arc<[u8]>,
    pub values: Vec<DataType>,
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
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = db.deque_push(self.into())?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
