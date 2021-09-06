use db::Db;
use macros::ParseFrames;

use crate::{frame_parse::data_type_to_frame, Frame};

/// <https://redis.io/commands/hgetall>
#[derive(Debug, ParseFrames)]
pub struct Hgetall {
    pub key: Box<[u8]>,
}

impl<'a> From<&'a Hgetall> for dict::cmd::kvp::get_all::Req<'a> {
    fn from(old: &'a Hgetall) -> Self {
        Self { key: &old.key }
    }
}

impl Hgetall {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let v = db.kvp_get_all((&self).into())?;
        Ok(Frame::Array(
            v.into_iter()
                .flat_map(|(k, v)| vec![Frame::Bulk(k), data_type_to_frame(v)].into_iter())
                .collect(),
        ))
    }
}
