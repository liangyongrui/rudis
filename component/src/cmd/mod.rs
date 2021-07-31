mod base;
mod hash;
mod list;
mod set;
mod sorted_set;

pub const SYNC_SNAPSHOT: &[u8] = b"*1\r\n$12\r\nsyncsnapshot\r\n";
pub const SYNC_CMD: &[u8] = b"*1\r\n$7\r\nsynccmd\r\n";
pub const SYNC_CMD_PING: &[u8] = b"*1\r\n$11\r\nsynccmdping\r\n";

use self::{
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
        zrank::Zrank, zrem::Zrem, zremrangebylex::Zremrangebylex, zremrangebyrank::Zremrangebyrank,
        zremrangebyscore::Zremrangebyscore, zrevrange::Zrevrange, zrevrangebylex::Zrevrangebylex,
        zrevrangebyscore::Zrevrangebyscore, zrevrank::Zrevrank,
    },
};
use crate::{Db, Frame, Parse, ParseError};

/// Enumeration of supported Redis commands.
///
/// Methods called on `Command` are delegated to the command implementation.
#[derive(Debug)]
pub enum Command {
    ReadCmd(ReadCmd),
    WriteCmd(WriteCmd),
    SyncCmd,
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
    Zremrangebylex(Zremrangebylex),
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
            "zremrangebylex" => Command::WriteCmd(WriteCmd::Zremrangebylex(
                Zremrangebylex::parse_frames(&mut parse)?,
            )),
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
            "synccmd" => Command::SyncCmd,
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
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        match self {
            Command::ReadCmd(cmd) => cmd.apply(db),
            Command::WriteCmd(cmd) => cmd.apply(db),
            Command::Unknown(cmd) => cmd.apply(),
            Command::SyncCmd => todo!(),
        }
    }
}

impl WriteCmd {
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        use WriteCmd::*;
        match self {
            Set(cmd) => cmd.apply(db),
            Psetex(cmd) => cmd.apply(db),
            Setex(cmd) => cmd.apply(db),
            Del(cmd) => cmd.apply(db),
            Pexpireat(cmd) => cmd.apply(db),
            Expireat(cmd) => cmd.apply(db),
            Expire(cmd) => cmd.apply(db),
            Pexpire(cmd) => cmd.apply(db),
            Incrby(cmd) => cmd.apply(db),
            Incr(cmd) => cmd.apply(db),
            Decr(cmd) => cmd.apply(db),
            Decrby(cmd) => cmd.apply(db),
            Lpush(cmd) => cmd.apply(db),
            Rpush(cmd) => cmd.apply(db),
            Lpushx(cmd) => cmd.apply(db),
            Rpushx(cmd) => cmd.apply(db),
            Lpop(cmd) => cmd.apply(db),
            Rpop(cmd) => cmd.apply(db),
            Hset(cmd) => cmd.apply(db),
            Hdel(cmd) => cmd.apply(db),
            Hsetnx(cmd) => cmd.apply(db),
            Hincrby(cmd) => cmd.apply(db),
            Sadd(cmd) => cmd.apply(db),
            Srem(cmd) => cmd.apply(db),
            Zadd(cmd) => cmd.apply(db),
            Zrem(cmd) => cmd.apply(db),
            Zremrangebyrank(cmd) => cmd.apply(db),
            Zremrangebyscore(cmd) => cmd.apply(db),
            Zremrangebylex(cmd) => cmd.apply(db),
        }
    }
}

impl ReadCmd {
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        use ReadCmd::*;

        match self {
            Get(cmd) => cmd.apply(db),
            Llen(cmd) => cmd.apply(db),
            Hgetall(cmd) => cmd.apply(db),
            Hget(cmd) => cmd.apply(db),
            Hmget(cmd) => cmd.apply(db),
            Hexists(cmd) => cmd.apply(db),
            Sismember(cmd) => cmd.apply(db),
            Smembers(cmd) => cmd.apply(db),
            Smismember(cmd) => cmd.apply(db),
            Zrangebylex(cmd) => cmd.apply(db),
            Zrangebyscore(cmd) => cmd.apply(db),
            Zrank(cmd) => cmd.apply(db),
            Zrevrange(cmd) => cmd.apply(db),
            Zrevrangebylex(cmd) => cmd.apply(db),
            Zrevrangebyscore(cmd) => cmd.apply(db),
            Zrevrank(cmd) => cmd.apply(db),
            Zrange(cmd) => cmd.apply(db),
            Lrange(cmd) => cmd.apply(db),
            Exists(cmd) => cmd.apply(db),
        }
    }
}
