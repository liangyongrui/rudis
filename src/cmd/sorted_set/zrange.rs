use std::ops::Bound;

use tracing::instrument;

use crate::{
    parse::ParseError,
    slot::data_type::{Float, SimpleType},
    utils::{other_type::ZrangeItem, BoundExt},
    Db, Frame, Parse,
};

enum By {
    Score,
    Lex,
    Rank,
}

/// https://redis.io/commands/zrange
#[derive(Debug)]
pub struct Zrange {
    pub key: SimpleType,
    pub range_item: ZrangeItem,
    pub rev: bool,
    pub limit: Option<(i64, i64)>,
    pub withscores: bool,
}

impl Zrange {
    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_simple_type()?;
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
                    Bound::Excluded(s.to_owned())
                } else {
                    Bound::Included(min[1..].to_owned())
                };
                let max = if max == "+" {
                    Bound::Unbounded
                } else if let Some(s) = max.strip_prefix('(') {
                    Bound::Excluded(s.to_owned())
                } else {
                    Bound::Included(max[1..].to_owned())
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

    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let limit = self
            .limit
            .map(|t| (if t.0 < 0 { 0 } else { t.0 as _ }, t.1));
        let key = &self.key;
        let rev = self.rev;
        let response = match self.range_item {
            ZrangeItem::Rank((start, stop)) => {
                let cmd = crate::slot::cmd::sorted_set::range_by_rank::Req {
                    key,
                    start,
                    stop,
                    limit,
                    rev,
                };
                db.sorted_set_range_by_rank(cmd)?
            }
            ZrangeItem::Socre((b, e)) => {
                let cmd = crate::slot::cmd::sorted_set::range_by_score::Req {
                    key,
                    rev,
                    range: (b.map(|f| Float(f)), e.map(|f| Float(f))),
                    limit,
                };
                db.sorted_set_range_by_score(cmd)?
            }
            ZrangeItem::Lex((b, e)) => {
                let cmd = crate::slot::cmd::sorted_set::range_by_lex::Req {
                    key,
                    rev,
                    range: (
                        b.map(|f| SimpleType::String(f.into())),
                        e.map(|f| SimpleType::String(f.into())),
                    ),
                    limit,
                };
                db.sorted_set_range_by_lex(cmd)?
            }
        };

        let mut res = vec![];
        for n in response {
            res.push((&n.key).into());
            if self.withscores {
                res.push(Frame::Simple(n.score.0.to_string()));
            }
        }
        Ok(Frame::Array(res))
    }
}
