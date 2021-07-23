use std::convert::TryInto;

use tracing::instrument;

use crate::{
    parse::ParseError,
    slot::data_type::{sorted_set::Node, SimpleType},
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
    pub nodes: Vec<Node>,
}

impl From<Zadd> for crate::slot::cmd::sorted_set::add::Req {
    fn from(old: Zadd) -> Self {
        Self {
            key: old.key,
            nodes: old.nodes,
            nx_xx: old.nx_xx,
            gt_lt: old.gt_lt,
            incr: old.incr,
        }
    }
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
                SimpleType::String(s) => {
                    let lowercase = s.to_lowercase();
                    match &lowercase[..] {
                        "nx" => nx_xx = NxXx::Nx,
                        "xx" => nx_xx = NxXx::Xx,
                        "gt" => gt_lt = GtLt::Gt,
                        "lt" => gt_lt = GtLt::Lt,
                        "incr" => incr = true,
                        "ch" => ch = true,
                        _ => break SimpleType::String(s),
                    }
                }
                t => break t,
            }
        };
        let member = parse.next_simple_type()?;
        let mut nodes = vec![Node::new(member, (&score).try_into()?)];
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
            nodes.push(Node::new(p[1].to_owned(), (&p[0]).try_into()?));
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
        // let mut res = vec![Frame::Simple("ZADD".to_owned()), self.key.into()];
        // match self.nx_xx {
        //     NxXx::Nx => res.push(Frame::Simple("NX".to_owned())),
        //     NxXx::Xx => res.push(Frame::Simple("XX".to_owned())),
        //     NxXx::None => (),
        // }
        // match self.gt_lt {
        //     GtLt::Gt => res.push(Frame::Simple("GT".to_owned())),
        //     GtLt::Lt => res.push(Frame::Simple("LT".to_owned())),
        //     GtLt::None => (),
        // }
        // if self.ch {
        //     res.push(Frame::Simple("CH".to_owned()))
        // }
        // if self.incr {
        //     res.push(Frame::Simple("INCR".to_owned()))
        // }
        // for node in self.nodes {
        //     res.push(Frame::Simple(node.score.to_string()));
        //     res.push(node.key.into());
        // }
        // Frame::Array(res).into()

        todo!()
    }

    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let ch = self.ch;
        let res = db.sorted_set_add(self.into()).await?;
        if ch {
            Ok(Frame::Integer(res.new_len as _))
        } else {
            Ok(Frame::Integer((res.new_len - res.old_len) as _))
        }
    }
}
