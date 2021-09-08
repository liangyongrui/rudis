use std::ops::Bound;

use common::other_type::LexRange;
use connection::parse::{frame::Frame, Parse, ParseError};
use db::Db;
use keys::Key;

/// <https://redis.io/commands/zrangebylex>
#[derive(Debug)]
pub struct Zrangebylex {
    pub key: Key,
    pub range_item: LexRange,
    pub limit: Option<(i64, i64)>,
}

impl Zrangebylex {
    pub fn parse_frames(parse: &mut Parse) -> common::Result<Self> {
        let key = parse.next_key()?;
        let min = parse.next_string()?;
        let max = parse.next_string()?;
        let mut limit = None;
        loop {
            let lowercase = match parse.next_string() {
                Ok(s) => s.to_lowercase(),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
            match &lowercase[..] {
                "limit" => limit = Some((parse.next_int()?, parse.next_int()?)),
                s => return Err(format!("unknown token: {}", s).into()),
            }
        }
        let range_item = {
            let min = if min == "-" {
                Bound::Unbounded
            } else if let Some(s) = min.strip_prefix('(') {
                Bound::Excluded(s.as_bytes().into())
            } else {
                Bound::Included(min[1..].as_bytes().into())
            };
            let max = if max == "+" {
                Bound::Unbounded
            } else if let Some(s) = max.strip_prefix('(') {
                Bound::Excluded(s.as_bytes().into())
            } else {
                Bound::Included(max[1..].as_bytes().into())
            };
            (min, max)
        };
        Ok(Self {
            key,
            range_item,
            limit,
        })
    }

    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let limit = self
            .limit
            .map(|t| (if t.0 < 0 { 0 } else { t.0 as _ }, t.1));
        let key = &self.key;
        let (b, e) = self.range_item;
        let cmd = dict::cmd::sorted_set::range_by_lex::Req {
            key,
            rev: false,
            range: (b, e),
            limit,
        };
        let response = db.sorted_set_range_by_lex(cmd)?;
        let mut res = vec![];
        for n in response {
            res.push(Frame::OwnedSimple(n.key.into()));
        }
        Ok(Frame::Array(res))
    }
}
