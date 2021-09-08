use common::options::NxXx;
use db::Db;
use dict::data_type::DataType;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/lpush>
#[derive(Debug, Clone, ParseFrames)]
pub struct Lpush {
    pub key: Key,
    pub values: Vec<DataType>,
}

impl From<Lpush> for dict::cmd::deque::push::Req {
    fn from(old: Lpush) -> Self {
        Self {
            key: old.key,
            left: true,
            elements: old.values,
            nx_xx: NxXx::None,
        }
    }
}

impl Lpush {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame<'_>> {
        let response = db.deque_push(self.into())?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
