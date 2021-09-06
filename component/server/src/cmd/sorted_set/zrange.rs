use std::ops::Bound;

use common::{float::Float, other_type::ZrangeItem, BoundExt};
use connection::parse::{frame::Frame, Parse, ParseError};
use db::Db;
use keys::Key;

enum By {
    Score,
    Lex,
    Rank,
}

/// <https://redis.io/commands/zrange>
#[derive(Debug)]
pub struct Zrange {
    pub key: Key,
    pub range_item: ZrangeItem,
    pub rev: bool,
    pub limit: Option<(i64, i64)>,
    pub withscores: bool,
}

impl Zrange {
    pub fn parse_frames(parse: &mut Parse) -> common::Result<Self> {
        let key = parse.next_key()?;
        let min = parse.next_string()?;
        let max = parse.next_string()?;
        let mut by = By::Rank;
        let mut rev = false;
        let mut limit = None;
        let mut withscores = false;
        loop {
            let lowercase = match parse.next_string() {
                Ok(s) => s.to_lowercase(),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
            match &lowercase[..] {
                "byscore" => by = By::Score,
                "bylex" => by = By::Lex,
                "limit" => limit = Some((parse.next_int()?, parse.next_int()?)),
                "rev" => rev = true,
                "withscores" => withscores = true,
                s => return Err(format!("unknown token: {}", s).into()),
            }
        }
        let range_item = match by {
            By::Score => {
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
                ZrangeItem::Socre((min, max))
            }
            By::Lex => {
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
                ZrangeItem::Lex((min, max))
            }
            By::Rank => ZrangeItem::Rank((min.parse()?, max.parse()?)),
        };
        Ok(Self {
            key,
            range_item,
            rev,
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
        let rev = self.rev;
        let response = match self.range_item {
            ZrangeItem::Rank((start, stop)) => {
                let cmd = dict::cmd::sorted_set::range_by_rank::Req {
                    key,
                    start,
                    stop,
                    limit,
                    rev,
                };
                db.sorted_set_range_by_rank(cmd)?
            }
            ZrangeItem::Socre((b, e)) => {
                let cmd = dict::cmd::sorted_set::range_by_score::Req {
                    key,
                    rev,
                    range: (b.map(Float), e.map(Float)),
                    limit,
                };
                db.sorted_set_range_by_score(cmd)?
            }
            ZrangeItem::Lex((b, e)) => {
                let cmd = dict::cmd::sorted_set::range_by_lex::Req {
                    key,
                    rev,
                    range: (b, e),
                    limit,
                };
                db.sorted_set_range_by_lex(cmd)?
            }
        };

        let mut res = vec![];
        for n in response {
            res.push(Frame::Simple(n.key));
            if self.withscores {
                res.push(Frame::Simple(n.score.0.to_string().as_bytes().into()));
            }
        }
        Ok(Frame::Array(res))
    }
}
