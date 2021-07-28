use std::ops::Bound;

use tracing::instrument;

use crate::{parse::ParseError, slot::data_type::Float, utils::BoundExt, Db, Frame, Parse};

/// https://redis.io/commands/zrevrangebyscore
#[derive(Debug)]
pub struct Zrevrangebyscore {
    pub key: Vec<u8>,
    pub range_item: (Bound<f64>, Bound<f64>),
    pub limit: Option<(i64, i64)>,
    pub withscores: bool,
}

impl Zrevrangebyscore {
    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
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

    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let limit = self
            .limit
            .map(|t| (if t.0 < 0 { 0 } else { t.0 as _ }, t.1));
        let key = &self.key;
        let (b, e) = self.range_item;
        let cmd = crate::slot::cmd::sorted_set::range_by_score::Req {
            key,
            rev: true,
            range: (e.map(Float), b.map(Float)),
            limit,
        };
        let response = db.sorted_set_range_by_score(cmd)?;

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
