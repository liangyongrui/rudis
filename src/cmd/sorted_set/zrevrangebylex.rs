use std::ops::Bound;

use tracing::instrument;

use crate::{
    db::data_type::{SimpleType, ZrangeItem},
    parse::ParseError,
    Db, Frame, Parse,
};

/// https://redis.io/commands/zrevrangebylex
#[derive(Debug)]
pub struct Zrevrangebylex {
    pub key: SimpleType,
    pub range_item: ZrangeItem,
    pub limit: Option<(i64, i64)>,
}

impl Zrevrangebylex {
    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_simple_type()?;
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
            ZrangeItem::Lex((min, max))
        };
        Ok(Self {
            key,
            range_item,
            limit,
        })
    }

    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.zrange(&self.key, self.range_item, true, self.limit) {
            Ok(v) => {
                let mut res = vec![];
                for n in v {
                    res.push(n.key.into());
                }
                Frame::Array(res)
            }
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
