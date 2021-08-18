use common::options::{GtLt, NxXx};
use db::Db;
use dict::cmd::simple::expire::Req;

use crate::{Frame, ParseError::EndOfStream};

/// https://redis.io/commands/pexpireat
///
/// 这个命令不知道是不是官方文档错了，文档上写的 \[NX|XX|GT|LT],
/// 我实现的是 \[NX|XX] 和 \[GT|LT] 每组可以指定0或1个
#[derive(Debug, Clone)]
pub struct Pexpireat {
    pub req: Req,
}

impl Pexpireat {
    pub fn parse_frames(parse: &mut crate::parse::Parse) -> common::Result<Pexpireat> {
        let key = parse.next_key()?;
        let expires_at = parse.next_int()?;
        let mut nx_xx = NxXx::None;
        let mut gt_lt = GtLt::None;
        loop {
            // Attempt to parse another string.
            match parse.next_string() {
                Ok(s) => match &s.to_uppercase()[..] {
                    "NX" => {
                        if !nx_xx.is_none() {
                            return Err("`NX` or `XX` already set".into());
                        }
                        nx_xx = NxXx::Nx
                    }
                    "XX" => {
                        if !nx_xx.is_none() {
                            return Err("`NX` or `XX` already set".into());
                        }
                        nx_xx = NxXx::Xx
                    }
                    "GT" => {
                        if !gt_lt.is_none() {
                            return Err("`GT` or `LT` already set".into());
                        }
                        gt_lt = GtLt::Gt
                    }
                    "LT" => {
                        if !gt_lt.is_none() {
                            return Err("`GT` or `LT` already set".into());
                        }
                        gt_lt = GtLt::Lt
                    }
                    not_support => return Err(format!("not support cmd: {}", not_support).into()),
                },
                Err(EndOfStream) => {
                    break;
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(Self {
            req: Req {
                key,
                expires_at: expires_at as u64,
                nx_xx,
                gt_lt,
            },
        })
    }

    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        let res = db.expire(self.req)?;
        let response = Frame::Integer(if res { 1 } else { 0 });
        Ok(response)
    }
}