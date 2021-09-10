use common::{connection::parse::frame::Frame, options::NxXx};
use db::Db;
use dict::data_type::DataType;
use keys::Key;
use macros::ParseFrames;

/// <https://redis.io/commands/hset>
#[derive(Debug, ParseFrames)]
pub struct Hset {
    pub key: Key,
    pub entries: Vec<(Box<[u8]>, DataType)>,
}

impl Hset {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.kvp_set(dict::cmd::kvp::set::Req {
            key: self.key,
            entries: self.entries,
            nx_xx: NxXx::None,
        })?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
