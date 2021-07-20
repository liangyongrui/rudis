use rcc_macros::ParseFrames;
use tracing::instrument;

use crate::{db::data_type::SimpleType, Db, Frame};
/// https://redis.io/commands/hgetall
#[derive(Debug, ParseFrames)]
pub struct Hgetall {
    pub key: SimpleType,
}

impl Hgetall {
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db.hgetall(&self.key) {
            Ok(v) => Frame::Array(
                v.into_iter()
                    .flat_map(|i| vec![i.key.into(), i.value.into()].into_iter())
                    .collect(),
            ),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
