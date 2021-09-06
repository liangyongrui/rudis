use db::Db;
use macros::ParseFrames;

use crate::{frame_parse, Frame};

/// https://redis.io/commands/hget
#[derive(Debug, ParseFrames)]
pub struct Hget {
    pub key: Box<[u8]>,
    pub field: Box<[u8]>,
}

impl<'a> From<&'a Hget> for dict::cmd::kvp::get::Req<'a> {
    fn from(old: &'a Hget) -> Self {
        Self {
            key: &old.key,
            fields: vec![&old.field],
        }
    }
}

impl Hget {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        Ok(frame_parse::ref_data_type_to_frame(
            &db.kvp_get((&self).into())?[0],
        ))
    }
}
