use std::convert::TryInto;

use tracing::instrument;

use crate::{
    db::data_type::{SimpleType, SortedSetNode},
    parse::ParseError,
    utils::options::{GtLt, NxXx},
    Db, Frame, Parse,
};

/// https://redis.io/commands/zadd
#[derive(Debug, Clone)]
pub struct Zadd {
    pub key: SimpleType,
    pub nx_xx: NxXx,
    pub gt_lt: GtLt,
    pub ch: bool,
    pub incr: bool,
    pub nodes: Vec<SortedSetNode>,
}

impl Zadd {
    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_simple_type()?;
        let mut nx_xx = NxXx::None;
        let mut gt_lt = GtLt::None;
        let mut ch = false;
        let mut incr = false;
        let score = loop {
            match parse.next_simple_type()? {
                SimpleType::SimpleString(s) => {
                    let lowercase = s.to_lowercase();
                    match &lowercase[..] {
                        "nx" => nx_xx = NxXx::Nx,
                        "xx" => nx_xx = NxXx::Xx,
                        "gt" => gt_lt = GtLt::Gt,
                        "lt" => gt_lt = GtLt::Lt,
                        "incr" => incr = true,
                        "ch" => ch = true,
                        _ => break SimpleType::SimpleString(s),
                    }
                }
                t => break t,
            }
        };
        let member = parse.next_simple_type()?;
        let mut nodes = vec![SortedSetNode::new(member, (&score).try_into()?)];
        let mut values = vec![];
        loop {
            match parse.next_simple_type() {
                Ok(s) => values.push(s),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
        }
        if values.len() % 2 != 0 {
            return Err(format!("参数数量错误: {}", values.len()).into());
        }
        for p in values.windows(2) {
            nodes.push(SortedSetNode::new(p[1].to_owned(), (&p[0]).try_into()?));
        }
        Ok(Self {
            key,
            nx_xx,
            gt_lt,
            ch,
            incr,
            nodes,
        })
    }

    pub fn into_cmd_bytes(self) -> Vec<u8> {
        let mut res = vec![Frame::Simple("ZADD".to_owned()), self.key.into()];
        match self.nx_xx {
            NxXx::Nx => res.push(Frame::Simple("NX".to_owned())),
            NxXx::Xx => res.push(Frame::Simple("XX".to_owned())),
            NxXx::None => (),
        }
        match self.gt_lt {
            GtLt::Gt => res.push(Frame::Simple("GT".to_owned())),
            GtLt::Lt => res.push(Frame::Simple("LT".to_owned())),
            GtLt::None => (),
        }
        if self.ch {
            res.push(Frame::Simple("CH".to_owned()))
        }
        if self.incr {
            res.push(Frame::Simple("INCR".to_owned()))
        }
        for node in self.nodes {
            res.push(Frame::Simple(node.score.to_string()));
            res.push(node.key.into());
        }
        Frame::Array(res).into()
    }

    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let response = match db
            .zadd(
                self.key, self.nodes, self.nx_xx, self.gt_lt, self.ch, self.incr,
            )
            .await
        {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        Ok(response)
    }
}
