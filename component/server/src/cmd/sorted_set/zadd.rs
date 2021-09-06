use std::convert::TryInto;

use common::options::{GtLt, NxXx};
use connection::parse::{frame::Frame, Parse, ParseError};
use db::Db;
use dict::data_type::{sorted_set::Node, DataType};
use keys::Key;
use tracing::debug;

use crate::frame_parse::next_data_type;

/// https://redis.io/commands/zadd
#[derive(Debug, Clone)]
pub struct Zadd {
    pub key: Key,
    pub nx_xx: NxXx,
    pub gt_lt: GtLt,
    pub ch: bool,
    pub incr: bool,
    pub nodes: Vec<Node>,
}

impl From<Zadd> for dict::cmd::sorted_set::add::Req {
    fn from(old: Zadd) -> Self {
        Self {
            key: old.key,
            nodes: old.nodes,
            nx_xx: old.nx_xx,
            gt_lt: old.gt_lt,
            incr: old.incr,
        }
    }
}

impl Zadd {
    pub fn parse_frames(parse: &mut Parse) -> common::Result<Self> {
        let key = parse.next_key()?;
        let mut nx_xx = NxXx::None;
        let mut gt_lt = GtLt::None;
        let mut ch = false;
        let mut incr = false;
        let score = loop {
            match next_data_type(parse)? {
                DataType::Bytes(avu) => {
                    let s = match std::str::from_utf8(&avu) {
                        Ok(s) => s,
                        Err(_) => break DataType::Bytes(avu),
                    };
                    let lowercase = s.to_lowercase();
                    match &lowercase[..] {
                        "nx" => nx_xx = NxXx::Nx,
                        "xx" => nx_xx = NxXx::Xx,
                        "gt" => gt_lt = GtLt::Gt,
                        "lt" => gt_lt = GtLt::Lt,
                        "incr" => incr = true,
                        "ch" => ch = true,
                        _ => break DataType::Bytes(avu),
                    }
                }
                t => break t,
            }
        };
        debug!(?score);
        let member = parse.next_bulk()?;
        let mut nodes = vec![Node::new(member, (&score).try_into()?)];
        loop {
            let score = match next_data_type(parse) {
                Ok(s) => (&s).try_into()?,
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
            let member = parse.next_bulk()?;
            nodes.push(Node::new(member, score));
        }

        Ok(Self {
            key,
            nx_xx,
            gt_lt,
            ch,
            incr,
            nodes,
        })
    }

    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let ch = self.ch;
        let res = db.sorted_set_add(self.into())?;
        if ch {
            Ok(Frame::Integer(res.new_len as _))
        } else {
            Ok(Frame::Integer((res.new_len - res.old_len) as _))
        }
    }
}
