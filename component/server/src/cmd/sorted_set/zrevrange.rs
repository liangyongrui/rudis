use connection::parse::{frame::Frame, Parse, ParseError};
use db::Db;
use keys::Key;

/// https://redis.io/commands/zrevrange
#[derive(Debug)]
pub struct Zrevrange {
    pub key: Key,
    pub range: (i64, i64),
    pub withscores: bool,
}

impl<'a> From<&'a Zrevrange> for dict::cmd::sorted_set::range_by_rank::Req<'a> {
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
    pub fn parse_frames(parse: &mut Parse) -> common::Result<Self> {
        let key = parse.next_key()?;
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

    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let withscores = self.withscores;
        let response = db.sorted_set_range_by_rank((&self).into())?;
        let mut res = vec![];
        if withscores {
            for n in response {
                res.push(Frame::Simple(n.key.into()));
                res.push(Frame::Simple(n.score.0.to_string().into()));
            }
        } else {
            for n in response {
                res.push(Frame::Simple(n.key.into()));
            }
        }
        Ok(Frame::Array(res))
    }
}
