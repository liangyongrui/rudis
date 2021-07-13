use tracing::{debug, instrument};

use crate::{
    db::data_type::{SimpleType, ZrangeItem},
    parse::ParseError,
    Connection, Db, Frame, Parse,
};

/// https://redis.io/commands/zrevrange
#[derive(Debug)]
pub struct Zrevrange {
    pub key: SimpleType,
    pub range: (i64, i64),
    pub withscores: bool,
}

impl Zrevrange {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
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

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.zrange(&self.key, ZrangeItem::Rank(self.range), true, None) {
            Ok(v) => {
                let mut res = vec![];
                for n in v {
                    res.push(n.key.into());
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
