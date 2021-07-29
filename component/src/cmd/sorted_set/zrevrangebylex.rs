use std::ops::Bound;

use tracing::instrument;

use crate::{parse::ParseError, utils::BoundExt, Db, Frame, Parse};

/// https://redis.io/commands/zrevrangebylex
#[derive(Debug)]
pub struct Zrevrangebylex {
    pub key: Vec<u8>,
    pub range_item: (Bound<String>, Bound<String>),
    pub limit: Option<(i64, i64)>,
}

impl Zrevrangebylex {
    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
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
            (min, max)
        };
        Ok(Self {
            key,
            range_item,
            limit,
        })
    }

    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let limit = self
            .limit
            .map(|t| (if t.0 < 0 { 0 } else { t.0 as _ }, t.1));
        let key = &self.key;
        let (b, e) = self.range_item;
        let cmd = crate::slot::cmd::sorted_set::range_by_lex::Req {
            key,
            rev: true,
            range: (e.map(|f| f.into()), b.map(|f| f.into())),
            limit,
        };
        let response = db.sorted_set_range_by_lex(cmd)?;
        let mut res = vec![];
        for n in response {
            res.push((&n.key).into());
        }
        Ok(Frame::Array(res))
    }
}
