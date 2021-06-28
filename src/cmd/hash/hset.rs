use std::convert::TryInto;

use tracing::{debug, instrument};

use crate::{
    db::data_type::{HashEntry, SimpleType},
    parse::ParseError,
    Connection, Db, Frame, Parse,
};

/// https://redis.io/commands/hset
#[derive(Debug)]
pub struct Hset {
    key: String,
    pairs: Vec<HashEntry>,
}

impl Hset {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let mut values = vec![];
        loop {
            match parse.next() {
                Ok(Frame::Simple(s)) => values.push(SimpleType::SimpleString(s)),
                Ok(Frame::Bulk(data)) => values.push(data.into()),
                Ok(Frame::Integer(data)) => values.push(data.into()),
                Ok(frame) => {
                    return Err(format!(
                        "protocol error; expected simple frame or bulk frame, got {:?}",
                        frame
                    )
                    .into())
                }
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
        }
        if values.len() % 2 != 0 {
            return Err(format!("参数数量错误: {}", values.len()).into());
        }
        let mut pairs = vec![];
        for p in values.windows(2) {
            pairs.push(HashEntry {
                field: p[0].clone().try_into()?,
                value: p[1].clone(),
            });
        }
        Ok(Self { key, pairs })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hset(self.key, self.pairs) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
