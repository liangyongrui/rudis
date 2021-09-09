use common::options::NxXx;
use db::Db;
use dict::data_type::DataType;
use keys::Key;
use macros::ParseFrames2;

use crate::Frame;

/// <https://redis.io/commands/rpush>
#[derive(Debug, Clone, ParseFrames2)]
pub struct Rpush {
    pub key: Key,
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
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = db.deque_push(self.into())?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
