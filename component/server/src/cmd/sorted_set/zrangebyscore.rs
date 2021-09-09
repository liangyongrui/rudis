use std::ops::Bound;

use common::{float::Float, BoundExt};
use connection::parse::{frame::Frame, Parse, ParseError};
use db::Db;
use keys::Key;

/// <https://redis.io/commands/zrangebyscore>
#[derive(Debug)]
pub struct Zrangebyscore {
    pub key: Key,
    pub range_item: (Bound<f64>, Bound<f64>),
    pub limit: Option<(i64, i64)>,
    pub withscores: bool,
}

impl Zrangebyscore {
    pub fn parse_frames(parse: &Parse) -> common::Result<Self> {
        let key = parse.next_key()?;
        let min = parse.next_string()?;
        let max = parse.next_string()?;
        let mut limit = None;
        let mut withscores = false;
        loop {
            let lowercase = match parse.next_string() {
                Ok(s) => s.to_lowercase(),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
            match &lowercase[..] {
                "limit" => limit = Some((parse.next_int()?, parse.next_int()?)),
                "withscores" => withscores = true,
                s => return Err(format!("unknown token: {}", s).into()),
            }
        }
        let range_item = {
            let min = if min == "-inf" {
                Bound::Unbounded
            } else if let Some(s) = min.strip_prefix('(') {
                Bound::Excluded(s.parse::<f64>()?)
            } else {
                Bound::Included(min.parse::<f64>()?)
            };
            let max = if max == "+inf" {
                Bound::Unbounded
            } else if let Some(s) = max.strip_prefix('(') {
                Bound::Excluded(s.parse::<f64>()?)
            } else {
                Bound::Included(max.parse::<f64>()?)
            };
            (min, max)
        };
        Ok(Self {
            key,
            range_item,
            limit,
            withscores,
        })
    }
    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let limit = self
            .limit
            .map(|t| (if t.0 < 0 { 0 } else { t.0 as _ }, t.1));
        let key = &self.key;
        let (b, e) = self.range_item;
        let cmd = dict::cmd::sorted_set::range_by_score::Req {
            key,
            rev: false,
            range: (b.map(Float), e.map(Float)),
            limit,
        };
        let response = db.sorted_set_range_by_score(cmd)?;

        let mut res = vec![];
        if self.withscores {
            for n in response {
                res.push(Frame::OwnedSimple(n.key.into()));
                res.push(Frame::OwnedStringSimple(n.score.0.to_string()));
            }
        } else {
            for n in response {
                res.push(Frame::OwnedSimple(n.key.into()));
            }
        }
        Ok(Frame::Array(res))
    }
}
