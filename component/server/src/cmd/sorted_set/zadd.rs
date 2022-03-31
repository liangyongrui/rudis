use common::{
    connection::parse::frame::Frame,
    float::Float,
    options::{GtLt, NxXx},
};
use db::Db;
use keys::Key;
use macros::ParseFrames;

/// <https://redis.io/commands/zadd>
#[derive(Debug, ParseFrames)]
pub struct Zadd {
    pub key: Key,
    #[optional]
    pub nx_xx: NxXx,
    #[optional]
    pub gt_lt: GtLt,
    pub ch: bool,
    pub incr: bool,
    pub nodes: Vec<(Float, Box<[u8]>)>,
}

impl Zadd {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let ch = self.ch;
        let res = db.sorted_set_add(dict::cmd::sorted_set::add::Req {
            key: self.key,
            nodes: self.nodes.into_iter().map(Into::into).collect(),
            nx_xx: self.nx_xx,
            gt_lt: self.gt_lt,
            incr: self.incr,
        })?;
        if ch {
            Ok(Frame::Integer(res.new_len as _))
        } else {
            Ok(Frame::Integer((res.new_len - res.old_len) as _))
        }
    }
}
