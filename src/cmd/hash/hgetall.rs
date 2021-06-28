use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame, Parse};

/// https://redis.io/commands/hgetall
#[derive(Debug)]
pub struct Hgetall {
    key: String,
}

impl Hgetall {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        Ok(Self { key })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.hgetall(&self.key) {
            Ok(v) => Frame::Array(
                v.into_iter()
                    .flat_map(|i| {
                        vec![SimpleType::SimpleString(i.field).into(), i.value.into()].into_iter()
                    })
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
