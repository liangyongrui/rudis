use std::ops::Bound;

use tracing::{debug, instrument};

use crate::{db::data_type::ZrangeItem, parse::ParseError, Connection, Db, Frame, Parse};

/// https://redis.io/commands/zrevrangebyscore
#[derive(Debug)]
pub struct Zrevrangebyscore {
    key: String,
    range_item: ZrangeItem,
    limit: Option<(i64, i64)>,
    withscores: bool,
}

impl Zrevrangebyscore {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
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
            ZrangeItem::Socre((min, max))
        };
        Ok(Self {
            key,
            range_item,
            limit,
            withscores,
        })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.zrange(&self.key, self.range_item, true, self.limit) {
            Ok(v) => {
                let mut res = vec![];
                for n in v {
                    res.push(Frame::Simple(n.key));
                    if self.withscores {
                        res.push(Frame::Simple(n.score.to_string()));
                    }
                }
                Frame::Array(res)
            }
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
