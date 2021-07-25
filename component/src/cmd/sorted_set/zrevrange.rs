use tracing::instrument;

use crate::{parse::ParseError, slot::data_type::SimpleType, Db, Frame, Parse};

/// https://redis.io/commands/zrevrange
#[derive(Debug)]
pub struct Zrevrange {
    pub key: SimpleType,
    pub range: (i64, i64),
    pub withscores: bool,
}

impl<'a> From<&'a Zrevrange> for crate::slot::cmd::sorted_set::range_by_rank::Req<'a> {
    fn from(old: &'a Zrevrange) -> Self {
        Self {
            key: &old.key,
            rev: true,
            start: old.range.0,
            stop: old.range.1,
            limit: None,
        }
    }
}
impl Zrevrange {
    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_simple_type()?;
        let min = parse.next_string()?;
        let max = parse.next_string()?;
        let mut withscores = false;
        loop {
            let lowercase = match parse.next_string() {
                Ok(s) => s.to_lowercase(),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
            match &lowercase[..] {
                "withscores" => withscores = true,
                s => return Err(format!("unknown token: {}", s).into()),
            }
        }
        Ok(Self {
            key,
            range: (min.parse()?, max.parse()?),
            withscores,
        })
    }

    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let withscores = self.withscores;
        let response = db.sorted_set_range_by_rank((&self).into())?;
        let mut res = vec![];
        for n in response {
            res.push((&n.key).into());
            if withscores {
                res.push(Frame::Simple(n.score.0.to_string()));
            }
        }
        Ok(Frame::Array(res))
    }
}
