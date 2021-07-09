use std::ops::Bound;

use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame, Parse};

/// https://redis.io/commands/zremrangebyrank
#[derive(Debug)]
pub struct Zremrangebyscore {
    key: SimpleType,
    range: (Bound<f64>, Bound<f64>),
}

impl Zremrangebyscore {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_simple_type()?;
        let min = parse.next_string()?;
        let max = parse.next_string()?;
        let range = {
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
            (min, max)
        };
        Ok(Self { key, range })
    }

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.zremrange_by_score(&self.key, self.range) {
            Ok(v) => Frame::Integer(v as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
