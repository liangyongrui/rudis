use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, parse::ParseError, Connection, Db, Frame, Parse};

/// https://redis.io/commands/smismember
#[derive(Debug)]
pub struct Smismember {
    key: String,
    values: Vec<SimpleType>,
}

impl Smismember {
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
        let response = match db.smismember(&self.key, self.values.iter().collect()) {
            Ok(i) => Frame::Array(
                i.into_iter()
                    .map(|t| if t { 1 } else { 0 })
                    .map(Frame::Integer)
                    .collect(),
            ),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
