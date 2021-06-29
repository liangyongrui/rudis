use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, parse::ParseError, Connection, Db, Frame, Parse};

/// https://redis.io/commands/sadd
#[derive(Debug)]
pub struct Sadd {
    key: String,
    values: Vec<SimpleType>,
}

impl Sadd {
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
        if values.is_empty() {
            return Err(ParseError::EndOfStream.into());
        }
        Ok(Self { key, values })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.sadd(self.key, self.values) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
