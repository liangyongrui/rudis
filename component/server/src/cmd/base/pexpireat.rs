use common::{
    connection::parse::frame::Frame,
    options::{GtLt, NxXx},
};
use db::Db;
use macros::ParseFrames;

/// <https://redis.io/commands/pexpireat>
///
/// 这个命令不知道是不是官方文档错了，文档上写的 \[NX|XX|GT|LT],
/// 我实现的是 \[NX|XX] 和 \[GT|LT] 每组可以指定0或1个
#[derive(Debug, ParseFrames)]
pub struct Pexpireat<'a> {
    pub key: &'a [u8],
    pub milliseconds_timestamp: u64,
    #[optional]
    pub nx_xx: NxXx,
    #[optional]
    pub gt_lt: GtLt,
}

impl Pexpireat<'_> {
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.expire(dict::cmd::simple::expire::Req {
            key: self.key.into(),
            expires_at: self.milliseconds_timestamp,
            nx_xx: self.nx_xx,
            gt_lt: self.gt_lt,
        })?;
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}
