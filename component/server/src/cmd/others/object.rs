use common::now_timestamp_ms;
use db::Db;
use macros::ParseFrames;

use crate::Frame;

#[derive(Debug, ParseFrames)]
pub struct Object<'a> {
    pub sub_cmd: &'a [u8],
    pub payload: Vec<&'a [u8]>,
}

impl Object<'_> {
    #[allow(clippy::unnecessary_wraps)]
    #[allow(unused_variables)]
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        // todo ignore case
        if self.sub_cmd == b"IDLETIME" || self.sub_cmd == b"idletime" {
            return if self.payload.is_empty() {
                Err("idletiem key not exists".into())
            } else if let Some(u) =
                db.get_last_visit_time(dict::cmd::simple::get_last_visit_time::Req {
                    key: self.payload[0],
                })?
            {
                #[allow(clippy::cast_possible_wrap)]
                Ok(Frame::Integer((now_timestamp_ms() / 1000 - u) as _))
            } else {
                Err("idletiem key not exists".into())
            };
        }
        // todo ignore case
        if self.sub_cmd == b"FREQ" || self.sub_cmd == b"freq" {
            return if self.payload.is_empty() {
                Err("freq key not exists".into())
            } else if let Some(u) = db.get_visit_times(dict::cmd::simple::get_visit_times::Req {
                key: self.payload[0],
            })? {
                #[allow(clippy::cast_possible_wrap)]
                Ok(Frame::Integer(u as _))
            } else {
                Err("freq key not exists".into())
            };
        }
        Ok(Frame::ok())
    }
}
