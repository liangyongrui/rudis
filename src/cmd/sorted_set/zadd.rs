use tracing::{debug, instrument};

use crate::{
    db::data_type::SortedSetNode,
    options::{GtLt, NxXx},
    parse::ParseError,
    Connection, Db, Frame, Parse,
};

/// https://redis.io/commands/hset
#[derive(Debug)]
pub struct Zadd {
    key: String,
    nx_xx: NxXx,
    gt_lt: GtLt,
    ch: bool,
    incr: bool,
    nodes: Vec<SortedSetNode>,
}

impl Zadd {
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Self> {
        let key = parse.next_string()?;
        let mut nx_xx = NxXx::None;
        let mut gt_lt = GtLt::None;
        let mut ch = false;
        let mut incr = false;
        let nk = loop {
            let lowercase = parse.next_string()?.to_lowercase();
            match &lowercase[..] {
                "nx" => nx_xx = NxXx::Nx,
                "xx" => nx_xx = NxXx::Xx,
                "gt" => gt_lt = GtLt::Gt,
                "lt" => gt_lt = GtLt::Lt,
                "incr" => incr = true,
                "ch" => ch = true,
                s => break s.to_owned(),
            }
        };
        let nv: f64 = parse.next_string()?.parse()?;

        let mut nodes = vec![SortedSetNode::new(nk, nv)];
        let mut values = vec![];
        loop {
            match parse.next_string() {
                Ok(s) => values.push(s),
                Err(ParseError::EndOfStream) => break,
                Err(err) => return Err(err.into()),
            };
        }
        if values.len() % 2 != 0 {
            return Err(format!("参数数量错误: {}", values.len()).into());
        }
        for p in values.windows(2) {
            nodes.push(SortedSetNode::new(p[0].to_owned(), p[1].parse()?));
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

    #[instrument(skip(self, db, dst))]
    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let response = match db.zadd(
            self.key, self.nodes, self.nx_xx, self.gt_lt, self.ch, self.incr,
        ) {
            Ok(i) => Frame::Integer(i as _),
            Err(e) => Frame::Error(e),
        };
        debug!(?response);
        // Write the response back to the client
        dst.write_frame(&response).await?;
        Ok(())
    }
}
