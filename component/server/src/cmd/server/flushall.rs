use std::sync::Arc;

use connection::parse::{Parse, ParseError};
use db::Db;

use crate::Frame;

/// <https://redis.io/commands/flushall>
#[derive(Debug, Clone)]
pub struct Flushall {
    pub sync: bool,
}

impl Flushall {
    pub fn parse_frames(parse: &mut Parse) -> common::Result<Flushall> {
        let mut sync = false;
        loop {
            #[allow(clippy::match_same_arms)]
            match parse.next_string() {
                Ok(s) if s.to_uppercase() == "SYNC" => sync = true,
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
                _ => break,
            };
        }
        Ok(Self { sync })
    }
    #[tracing::instrument(skip(db))]
    pub fn apply(self, db: Arc<Db>) -> Frame {
        db.flushall(self.sync);
        Frame::ok()
    }
}
