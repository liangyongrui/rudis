mod base;
mod hash;
mod list;
mod set;
mod sorted_set;

pub use self::{
    base::{
        decr::Decr, decrby::Decrby, del::Del, exists::Exists, expire::Expire, expireat::Expireat,
        get::Get, incr::Incr, incrby::Incrby, pexpire::Pexpire, pexpireat::Pexpireat,
        psetex::Psetex, set::Set, setex::Setex, unknown::Unknown,
    },
    hash::{
        hdel::Hdel, hexists::Hexists, hget::Hget, hgetall::Hgetall, hincrby::Hincrby, hmget::Hmget,
        hset::Hset, hsetnx::Hsetnx,
    },
    list::{
        llen::Llen, lpop::Lpop, lpush::Lpush, lpushx::Lpushx, lrange::Lrange, rpop::Rpop,
        rpush::Rpush, rpushx::Rpushx,
    },
    set::{
        sadd::Sadd, sismember::Sismember, smembers::Smembers, smismember::Smismember, srem::Srem,
    },
    sorted_set::{
        zadd::Zadd, zrange::Zrange, zrangebylex::Zrangebylex, zrangebyscore::Zrangebyscore,
        zrank::Zrank, zrem::Zrem, zremrangebyrank::Zremrangebyrank,
        zremrangebyscore::Zremrangebyscore, zrevrange::Zrevrange, zrevrangebylex::Zrevrangebylex,
        zrevrangebyscore::Zrevrangebyscore, zrevrank::Zrevrank,
    },
};
use crate::{Connection, Db, Frame, Parse, ParseError, Shutdown};

/// Enumeration of supported Redis commands.
///
/// Methods called on `Command` are delegated to the command implementation.
#[derive(Debug)]
pub enum Command {
    ReadCmd(ReadCmd),
    WriteCmd(WriteCmd),
    Unknown(Unknown),
}
#[derive(Debug)]
pub enum ReadCmd {
    Zrangebylex(Zrangebylex),
    Zrangebyscore(Zrangebyscore),
    Zrank(Zrank),
    Zrevrange(Zrevrange),
    Zrevrangebylex(Zrevrangebylex),
    Zrevrangebyscore(Zrevrangebyscore),
    Zrevrank(Zrevrank),
    Zrange(Zrange),
    Sismember(Sismember),
    Smembers(Smembers),
    Smismember(Smismember),
    Hexists(Hexists),
    Hget(Hget),
    Hmget(Hmget),
    Hgetall(Hgetall),
    Llen(Llen),
    Lrange(Lrange),
    Get(Get),
    Exists(Exists),
}

#[derive(Debug, Clone)]
pub enum WriteCmd {
    Zrem(Zrem),
    Zremrangebyrank(Zremrangebyrank),
    Zremrangebyscore(Zremrangebyscore),
    Zadd(Zadd),
    Sadd(Sadd),
    Srem(Srem),
    Hincrby(Hincrby),
    Hdel(Hdel),
    Hsetnx(Hsetnx),
    Hset(Hset),
    Lpop(Lpop),
    Rpop(Rpop),
    Lpush(Lpush),
    Rpush(Rpush),
    Lpushx(Lpushx),
    Rpushx(Rpushx),
    Incrby(Incrby),
    Incr(Incr),
    Decr(Decr),
    Decrby(Decrby),
    Set(Set),
    Del(Del),
    Psetex(Psetex),
    Setex(Setex),
    Pexpireat(Pexpireat),
    Expireat(Expireat),
    Expire(Expire),
    Pexpire(Pexpire),
}

impl Command {
    /// Parse a command from a received frame.
    ///
    /// The `Frame` must represent a Redis command supported by `rcc` and
    /// be the array variant.
    ///
    /// # Returns
    ///
    /// On success, the command value is returned, otherwise, `Err` is returned.
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        // The frame  value is decorated with `Parse`. `Parse` provides a
        // "cursor" like API which makes parsing the command easier.
        //
        // The frame value must be an array variant. Any other frame variants
        // result in an error being returned.
        let mut parse = Parse::new(frame)?;

        // All redis commands begin with the command name as a string. The name
        // is read and converted to lower cases in order to do case sensitive
        // matching.
        let command_name = parse.next_string()?.to_lowercase();

        // Match the command name, delegating the rest of the parsing to the
        // specific command.
        let command = match &command_name[..] {
            "zrangebylex" => {
                Command::ReadCmd(ReadCmd::Zrangebylex(Zrangebylex::parse_frames(&mut parse)?))
            }
            "zrangebyscore" => Command::ReadCmd(ReadCmd::Zrangebyscore(
                Zrangebyscore::parse_frames(&mut parse)?,
            )),
            "zrank" => Command::ReadCmd(ReadCmd::Zrank(Zrank::parse_frames(&mut parse)?)),
            "zrem" => Command::WriteCmd(WriteCmd::Zrem(Zrem::parse_frames(&mut parse)?)),
            "zremrangebyrank" => Command::WriteCmd(WriteCmd::Zremrangebyrank(
                Zremrangebyrank::parse_frames(&mut parse)?,
            )),
            "zremrangebyscore" => Command::WriteCmd(WriteCmd::Zremrangebyscore(
                Zremrangebyscore::parse_frames(&mut parse)?,
            )),
            "zrevrange" => {
                Command::ReadCmd(ReadCmd::Zrevrange(Zrevrange::parse_frames(&mut parse)?))
            }
            "zrevrangebylex" => Command::ReadCmd(ReadCmd::Zrevrangebylex(
                Zrevrangebylex::parse_frames(&mut parse)?,
            )),
            "zrevrangebyscore" => Command::ReadCmd(ReadCmd::Zrevrangebyscore(
                Zrevrangebyscore::parse_frames(&mut parse)?,
            )),
            "zrevrank" => Command::ReadCmd(ReadCmd::Zrevrank(Zrevrank::parse_frames(&mut parse)?)),
            "zrange" => Command::ReadCmd(ReadCmd::Zrange(Zrange::parse_frames(&mut parse)?)),
            "zadd" => Command::WriteCmd(WriteCmd::Zadd(Zadd::parse_frames(&mut parse)?)),
            "sadd" => Command::WriteCmd(WriteCmd::Sadd(Sadd::parse_frames(&mut parse)?)),
            "sismember" => {
                Command::ReadCmd(ReadCmd::Sismember(Sismember::parse_frames(&mut parse)?))
            }
            "smismember" => {
                Command::ReadCmd(ReadCmd::Smismember(Smismember::parse_frames(&mut parse)?))
            }
            "smembers" => Command::ReadCmd(ReadCmd::Smembers(Smembers::parse_frames(&mut parse)?)),
            "srem" => Command::WriteCmd(WriteCmd::Srem(Srem::parse_frames(&mut parse)?)),
            "hincrby" => Command::WriteCmd(WriteCmd::Hincrby(Hincrby::parse_frames(&mut parse)?)),
            "hexist" => Command::ReadCmd(ReadCmd::Hexists(Hexists::parse_frames(&mut parse)?)),
            "hdel" => Command::WriteCmd(WriteCmd::Hdel(Hdel::parse_frames(&mut parse)?)),
            "hsetnx" => Command::WriteCmd(WriteCmd::Hsetnx(Hsetnx::parse_frames(&mut parse)?)),
            "hget" => Command::ReadCmd(ReadCmd::Hget(Hget::parse_frames(&mut parse)?)),
            "hmget" => Command::ReadCmd(ReadCmd::Hmget(Hmget::parse_frames(&mut parse)?)),
            "hset" => Command::WriteCmd(WriteCmd::Hset(Hset::parse_frames(&mut parse)?)),
            "hgetall" => Command::ReadCmd(ReadCmd::Hgetall(Hgetall::parse_frames(&mut parse)?)),
            "llen" => Command::ReadCmd(ReadCmd::Llen(Llen::parse_frames(&mut parse)?)),
            "rpop" => Command::WriteCmd(WriteCmd::Rpop(Rpop::parse_frames(&mut parse)?)),
            "lpop" => Command::WriteCmd(WriteCmd::Lpop(Lpop::parse_frames(&mut parse)?)),
            "lrange" => Command::ReadCmd(ReadCmd::Lrange(Lrange::parse_frames(&mut parse)?)),
            "lpush" => Command::WriteCmd(WriteCmd::Lpush(Lpush::parse_frames(&mut parse)?)),
            "rpush" => Command::WriteCmd(WriteCmd::Rpush(Rpush::parse_frames(&mut parse)?)),
            "lpushx" => Command::WriteCmd(WriteCmd::Lpushx(Lpushx::parse_frames(&mut parse)?)),
            "rpushx" => Command::WriteCmd(WriteCmd::Rpushx(Rpushx::parse_frames(&mut parse)?)),
            "incrby" => Command::WriteCmd(WriteCmd::Incrby(Incrby::parse_frames(&mut parse)?)),
            "incr" => Command::WriteCmd(WriteCmd::Incr(Incr::parse_frames(&mut parse)?)),
            "decrby" => Command::WriteCmd(WriteCmd::Decrby(Decrby::parse_frames(&mut parse)?)),
            "decr" => Command::WriteCmd(WriteCmd::Decr(Decr::parse_frames(&mut parse)?)),
            "get" => Command::ReadCmd(ReadCmd::Get(Get::parse_frames(&mut parse)?)),
            "set" => Command::WriteCmd(WriteCmd::Set(Set::parse_frames(&mut parse)?)),
            "del" => Command::WriteCmd(WriteCmd::Del(Del::parse_frames(&mut parse)?)),
            "exists" => Command::ReadCmd(ReadCmd::Exists(Exists::parse_frames(&mut parse)?)),
            "psetex" => Command::WriteCmd(WriteCmd::Psetex(Psetex::parse_frames(&mut parse)?)),
            "setex" => Command::WriteCmd(WriteCmd::Setex(Setex::parse_frames(&mut parse)?)),
            "pexpireat" => {
                Command::WriteCmd(WriteCmd::Pexpireat(Pexpireat::parse_frames(&mut parse)?))
            }
            "expireat" => {
                Command::WriteCmd(WriteCmd::Expireat(Expireat::parse_frames(&mut parse)?))
            }
            "expire" => Command::WriteCmd(WriteCmd::Expire(Expire::parse_frames(&mut parse)?)),
            "pexpire" => Command::WriteCmd(WriteCmd::Pexpire(Pexpire::parse_frames(&mut parse)?)),
            _ => {
                // The command is not recognized and an Unknown command is
                // returned.
                //
                // `return` is called here to skip the `finish()` call below. As
                // the command is not recognized, there is most likely
                // unconsumed fields remaining in the `Parse` instance.
                return Ok(Command::Unknown(Unknown::new(command_name)));
            }
        };

        // Check if there is any remaining unconsumed fields in the `Parse`
        // value. If fields remain, this indicates an unexpected frame format
        // and an error is returned.
        parse.finish()?;

        // The command has been successfully parsed
        Ok(command)
    }

    /// Apply the command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    pub async fn apply(
        self,
        db: &Db,
        dst: &mut Connection,
        shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        match self {
            Command::ReadCmd(cmd) => cmd.apply(db, dst, shutdown).await,
            Command::WriteCmd(cmd) => cmd.apply(db, dst, shutdown).await,
            Command::Unknown(cmd) => cmd.apply(dst).await,
        }
    }
}

impl WriteCmd {
    pub async fn apply(
        self,
        db: &Db,
        dst: &mut Connection,
        _shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        use WriteCmd::*;
        if let Some(ref sender) = db.aof_sender {
            sender.send(self.clone()).await?;
        }
        match self {
            Set(cmd) => cmd.apply(db, dst).await,
            Psetex(cmd) => cmd.apply(db, dst).await,
            Setex(cmd) => cmd.apply(db, dst).await,
            Del(cmd) => cmd.apply(db, dst).await,
            Pexpireat(cmd) => cmd.apply(db, dst).await,
            Expireat(cmd) => cmd.apply(db, dst).await,
            Expire(cmd) => cmd.apply(db, dst).await,
            Pexpire(cmd) => cmd.apply(db, dst).await,
            Incrby(cmd) => cmd.apply(db, dst).await,
            Incr(cmd) => cmd.apply(db, dst).await,
            Decr(cmd) => cmd.apply(db, dst).await,
            Decrby(cmd) => cmd.apply(db, dst).await,
            Lpush(cmd) => cmd.apply(db, dst).await,
            Rpush(cmd) => cmd.apply(db, dst).await,
            Lpushx(cmd) => cmd.apply(db, dst).await,
            Rpushx(cmd) => cmd.apply(db, dst).await,
            Lpop(cmd) => cmd.apply(db, dst).await,
            Rpop(cmd) => cmd.apply(db, dst).await,
            Hset(cmd) => cmd.apply(db, dst).await,
            Hdel(cmd) => cmd.apply(db, dst).await,
            Hsetnx(cmd) => cmd.apply(db, dst).await,
            Hincrby(cmd) => cmd.apply(db, dst).await,
            Sadd(cmd) => cmd.apply(db, dst).await,
            Srem(cmd) => cmd.apply(db, dst).await,
            Zadd(cmd) => cmd.apply(db, dst).await,
            Zrem(cmd) => cmd.apply(db, dst).await,
            Zremrangebyrank(cmd) => cmd.apply(db, dst).await,
            Zremrangebyscore(cmd) => cmd.apply(db, dst).await,
        }
    }

    pub fn into_cmd_bytes(self) -> Vec<u8> {
        match self {
            WriteCmd::Zrem(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Zremrangebyrank(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Zremrangebyscore(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Zadd(cmd) => todo!(),
            WriteCmd::Sadd(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Srem(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Hincrby(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Hdel(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Hsetnx(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Hset(cmd) => todo!(),
            WriteCmd::Lpop(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Rpop(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Lpush(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Rpush(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Lpushx(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Rpushx(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Incrby(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Incr(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Decr(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Decrby(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Set(cmd) => todo!(),
            WriteCmd::Del(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Psetex(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Setex(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Pexpireat(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Expireat(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Expire(cmd) => cmd.into_cmd_bytes(),
            WriteCmd::Pexpire(cmd) => cmd.into_cmd_bytes(),
        }
    }
}

impl ReadCmd {
    pub async fn apply(
        self,
        db: &Db,
        dst: &mut Connection,
        _shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        use ReadCmd::*;

        match self {
            Get(cmd) => cmd.apply(db, dst).await,
            Llen(cmd) => cmd.apply(db, dst).await,
            Hgetall(cmd) => cmd.apply(db, dst).await,
            Hget(cmd) => cmd.apply(db, dst).await,
            Hmget(cmd) => cmd.apply(db, dst).await,
            Hexists(cmd) => cmd.apply(db, dst).await,
            Sismember(cmd) => cmd.apply(db, dst).await,
            Smembers(cmd) => cmd.apply(db, dst).await,
            Smismember(cmd) => cmd.apply(db, dst).await,
            Zrangebylex(cmd) => cmd.apply(db, dst).await,
            Zrangebyscore(cmd) => cmd.apply(db, dst).await,
            Zrank(cmd) => cmd.apply(db, dst).await,
            Zrevrange(cmd) => cmd.apply(db, dst).await,
            Zrevrangebylex(cmd) => cmd.apply(db, dst).await,
            Zrevrangebyscore(cmd) => cmd.apply(db, dst).await,
            Zrevrank(cmd) => cmd.apply(db, dst).await,
            Zrange(cmd) => cmd.apply(db, dst).await,
            Lrange(cmd) => cmd.apply(db, dst).await,
            Exists(cmd) => cmd.apply(db, dst).await,
        }
    }
}
