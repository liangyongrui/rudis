use std::ops::Bound;

use tracing::instrument;

use crate::{
    slot::data_type::{Float, SimpleType},
    utils::BoundExt,
    Db, Frame, Parse,
};

/// https://redis.io/commands/zremrangebyscore
#[derive(Debug, Clone)]
pub struct Zremrangebyscore {
    pub key: SimpleType,
    pub range: (Bound<f64>, Bound<f64>),
}

impl From<Zremrangebyscore> for crate::slot::cmd::sorted_set::remove_by_score_range::Req {
    fn from(old: Zremrangebyscore) -> Self {
        Self {
            key: old.key,
            rev: false,
            range: (old.range.0.map(|f| Float(f)), old.range.1.map(|f| Float(f))),
        }
    }
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
        // let mut res = vec![
        //     Frame::Simple("ZREMRANGEBYSCORE".to_owned()),
        //     self.key.into(),
        // ];
        // let bf64_to_frame = |b: Bound<f64>, left| match b {
        //     Bound::Included(a) => Frame::Simple(a.to_string()),
        //     Bound::Excluded(a) => Frame::Simple(format!("({}", a)),
        //     Bound::Unbounded => Frame::Simple(format!("{}inf", if left { "+" } else { "-" })),
        // };
        // res.push(bf64_to_frame(self.range.0, true));
        // res.push(bf64_to_frame(self.range.1, false));
        // Frame::Array(res).into()
        todo!()
    }
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let res = db.sorted_set_remove_by_score_range(self.into()).await?;
        Ok(Frame::Integer(res.len() as _))
    }
}
