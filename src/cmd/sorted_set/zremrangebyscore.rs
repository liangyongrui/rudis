use std::ops::Bound;

use tracing::{debug, instrument};

use crate::{db::data_type::SimpleType, Connection, Db, Frame, Parse};

/// https://redis.io/commands/zremrangebyscore
#[derive(Debug, Clone)]
pub struct Zremrangebyscore {
    pub key: SimpleType,
    pub range: (Bound<f64>, Bound<f64>),
}

impl Zremrangebyscore {
    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
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

    pub fn into_cmd_bytes(self) -> Vec<u8> {
        let mut res = vec![
            Frame::Simple("ZREMRANGEBYSCORE".to_owned()),
            self.key.into(),
        ];
        let bf64_to_frame = |b: Bound<f64>, left| match b {
            Bound::Included(a) => Frame::Simple(a.to_string()),
            Bound::Excluded(a) => Frame::Simple(format!("({}", a)),
            Bound::Unbounded => Frame::Simple(format!("{}inf", if left { "+" } else { "-" })),
        };
        res.push(bf64_to_frame(self.range.0, true));
        res.push(bf64_to_frame(self.range.1, false));
        Frame::Array(res).into()
    }
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
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
