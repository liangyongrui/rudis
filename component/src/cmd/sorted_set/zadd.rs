use std::{convert::TryInto, sync::Arc};

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
    pub key: Arc<[u8]>,
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
        let key = parse.next_key()?;
        let mut nx_xx = NxXx::None;
        let mut gt_lt = GtLt::None;
        let mut ch = false;
        let mut incr = false;
        let score = loop {
            match parse.next_simple_type()? {
                SimpleType::Bytes(avu) => {
                    let s = match std::str::from_utf8(&avu) {
                        Ok(s) => s,
                        Err(_) => break SimpleType::Bytes(avu),
                    };
                    let lowercase = s.to_lowercase();
                    match &lowercase[..] {
                        "nx" => nx_xx = NxXx::Nx,
                        "xx" => nx_xx = NxXx::Xx,
                        "gt" => gt_lt = GtLt::Gt,
                        "lt" => gt_lt = GtLt::Lt,
                        "incr" => incr = true,
                        "ch" => ch = true,
                        _ => break SimpleType::Bytes(avu),
                    }
                }
                t => break t,
            }
        };
        let member = parse.next_string()?;
        let mut nodes = vec![Node::new(member, (&score).try_into()?)];
        loop {
            nodes.push(Node::new(
                parse.next_string()?,
                match parse.next_simple_type() {
                    Ok(s) => (&s).try_into()?,
                    Err(ParseError::EndOfStream) => break,
                    Err(err) => return Err(err.into()),
                },
            ));
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

    #[instrument(skip(self, db))]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let ch = self.ch;
        let res = db.sorted_set_add(self.into())?;
        if ch {
            Ok(Frame::Integer(res.new_len as _))
        } else {
            Ok(Frame::Integer((res.new_len - res.old_len) as _))
        }
    }
}
