use common::options::NxXx;
use db::Db;
use dict::data_type::DataType;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/lpushx>
#[derive(Debug, ParseFrames)]
pub struct Lpushx {
    pub key: Key,
    pub values: Vec<DataType>,
}

impl From<Lpushx> for dict::cmd::deque::push::Req {
    #[inline]
    fn from(old: Lpushx) -> Self {
        Self {
            key: old.key,
            left: true,
            elements: old.values,
            nx_xx: NxXx::Xx,
        }
    }
}

impl Lpushx {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = db.deque_push(self.into())?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
