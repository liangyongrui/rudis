use common::options::NxXx;
use db::Db;
use dict::data_type::DataType;
use keys::Key;
use macros::ParseFrames;

use crate::Frame;

/// <https://redis.io/commands/rpushx>
#[derive(Debug, ParseFrames)]
pub struct Rpushx {
    pub key: Key,
    pub values: Vec<DataType>,
}

impl From<Rpushx> for dict::cmd::deque::push::Req {
    fn from(old: Rpushx) -> Self {
        Self {
            key: old.key,
            left: false,
            elements: old.values,
            nx_xx: NxXx::Xx,
        }
    }
}

impl Rpushx {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let response = db.deque_push(self.into())?;
        Ok(Frame::Integer(response.new_len as _))
    }
}
