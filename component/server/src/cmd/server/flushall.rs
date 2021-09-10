use std::sync::Arc;

use common::connection::parse::frame::Frame;
use db::Db;
use macros::ParseFrames;

/// <https://redis.io/commands/flushall>
#[derive(Debug, ParseFrames)]
pub struct Flushall {
    pub sync: bool,
}

impl Flushall {
    #[tracing::instrument(skip(db))]
    pub fn apply(self, db: Arc<Db>) -> Frame<'static> {
        db.flushall(self.sync);
        Frame::ok()
    }
}
