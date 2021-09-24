use db::Db;
use macros::ParseFrames;

use crate::Frame;

#[derive(Debug, ParseFrames)]
pub struct Config<'a> {
    pub sub_cmd: &'a [u8],
    pub payload: Vec<&'a [u8]>,
}

impl Config<'_> {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(unused_variables)]
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        Ok(Frame::ok())
    }
}
