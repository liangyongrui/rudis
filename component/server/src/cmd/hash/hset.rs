use std::sync::Arc;

use common::options::NxXx;
use connection::parse::{Parse, ParseError};
use db::Db;
use dict::data_type::DataType;

use crate::{frame_parse::next_data_type, Frame};

/// https://redis.io/commands/hset
#[derive(Debug, Clone)]
pub struct Hset {
    pub key: Arc<[u8]>,
    pub entries: Vec<(Arc<[u8]>, DataType)>,
}

impl From<Hset> for dict::cmd::kvp::set::Req {
    fn from(old: Hset) -> Self {
        Self {
            key: old.key,
            entries: old.entries,
            nx_xx: NxXx::None,
        }
    }
}

impl Hset {
    pub fn parse_frames(parse: &mut Parse) -> common::Result<Hset> {
        // Read the key to set. This is a required field
        let key = parse.next_key()?;

        let mut entries = vec![(parse.next_bulk()?, next_data_type(parse)?)];
        loop {
            match parse.next_bulk() {
                Ok(s) => entries.push((s, next_data_type(parse)?)),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
        }
        Ok(Self { key, entries })
    }

    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.kvp_set(self.into())?;
        Ok(Frame::Integer((res.new_len - res.old_len) as _))
    }
}
