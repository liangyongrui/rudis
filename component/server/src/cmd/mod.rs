mod base;
mod hash;
mod list;
/// <https://redis.io/commands#server>
mod server;
mod set;
mod sorted_set;
mod syncsnapshot;

use std::sync::Arc;

use common::connection::parse::{frame::Frame, Parse};
use db::Db;

use self::{
    base::{
        decr::Decr, decrby::Decrby, del::Del, exists::Exists, expire::Expire, expireat::Expireat,
        get::Get, incr::Incr, incrby::Incrby, pexpire::Pexpire, pexpireat::Pexpireat,
        psetex::Psetex, pttl::Pttl, set::Set, setex::Setex, ttl::Ttl, unknown::Unknown,
    },
    hash::{
        hdel::Hdel, hexists::Hexists, hget::Hget, hgetall::Hgetall, hincrby::Hincrby, hmget::Hmget,
        hset::Hset, hsetnx::Hsetnx,
    },
    list::{
        llen::Llen, lpop::Lpop, lpush::Lpush, lpushx::Lpushx, lrange::Lrange, rpop::Rpop,
        rpush::Rpush, rpushx::Rpushx,
    },
    server::{dump::Dump, flushall::Flushall, info::Info},
    set::{
        sadd::Sadd, sismember::Sismember, smembers::Smembers, smismember::Smismember, srem::Srem,
    },
    sorted_set::{
        zadd::Zadd, zrange::Zrange, zrangebylex::Zrangebylex, zrangebyscore::Zrangebyscore,
        zrank::Zrank, zrem::Zrem, zremrangebylex::Zremrangebylex, zremrangebyrank::Zremrangebyrank,
        zremrangebyscore::Zremrangebyscore, zrevrange::Zrevrange, zrevrangebylex::Zrevrangebylex,
        zrevrangebyscore::Zrevrangebyscore, zrevrank::Zrevrank,
    },
    syncsnapshot::SyncSnapshot,
};

/// Enumeration of supported Redis commands.
///
/// Methods called on `Command` are delegated to the command implementation.
#[derive(Debug)]
pub enum Command<'a> {
    Ping,
    Read(Read<'a>),
    Write(Write<'a>),
    SyncSnapshot(SyncSnapshot),
    Unknown(Unknown<'a>),
}
#[derive(Debug)]
pub enum Read<'a> {
    Zrangebylex(Zrangebylex),
    Zrangebyscore(Zrangebyscore),
    Zrank(Zrank<'a>),
    Zrevrange(Zrevrange),
    Zrevrangebylex(Zrevrangebylex),
    Zrevrangebyscore(Zrevrangebyscore),
    Zrevrank(Zrevrank<'a>),
    Zrange(Zrange),
    Sismember(Sismember<'a>),
    Smembers(Smembers<'a>),
    Smismember(Smismember<'a>),
    Hexists(Hexists<'a>),
    Hget(Hget<'a>),
    Hmget(Hmget<'a>),
    Hgetall(Hgetall<'a>),
    Llen(Llen<'a>),
    Lrange(Lrange<'a>),
    Get(Get<'a>),
    Ttl(Ttl<'a>),
    Pttl(Pttl<'a>),
    Exists(Exists<'a>),
    Info(Info),
    Dump(Dump<'a>),
}

#[derive(Debug, Clone)]
pub enum Write<'a> {
    Zrem(Zrem),
    Zremrangebyrank(Zremrangebyrank<'a>),
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
    Pexpireat(Pexpireat<'a>),
    Expireat(Expireat<'a>),
    Expire(Expire<'a>),
    Pexpire(Pexpire<'a>),
    Flushall(Flushall),
}

impl<'a> Command<'a> {
    /// Parse a command from a received frame.
    ///
    /// The `Frame` must represent a Redis command supported by `rudis` and
    /// be the array variant.
    ///
    /// # Returns
    ///
    /// On success, the command value is returned, otherwise, `Err` is returned.
    pub fn from_frame(frame: Frame<'a>) -> common::Result<Self> {
        // The frame  value is decorated with `Parse`. `Parse` provides a
        // "cursor" like API which makes parsing the command easier.
        //
        // The frame value must be an array variant. Any other frame variants
        // result in an error being returned.
        let parse = Parse::new(frame)?;

        // All redis commands begin with the command name as a string. The name
        // is read and converted to lower cases in order to do case sensitive
        // matching.
        let command_name = parse.next_string()?.to_lowercase();

        let parse_ref = unsafe { &*(&parse as *const _) };

        // Match the command name, delegating the rest of the parsing to the
        // specific command.
        let command = match &command_name[..] {
            "ping" => Command::Ping,
            "ttl" => Command::Read(Read::Ttl(Ttl::parse_frames(parse_ref)?)),
            "pttl" => Command::Read(Read::Pttl(Pttl::parse_frames(parse_ref)?)),
            "zrangebylex" => {
                Command::Read(Read::Zrangebylex(Zrangebylex::parse_frames(parse_ref)?))
            }
            "zrangebyscore" => {
                Command::Read(Read::Zrangebyscore(Zrangebyscore::parse_frames(parse_ref)?))
            }
            "zrank" => Command::Read(Read::Zrank(Zrank::parse_frames(parse_ref)?)),
            "zrem" => Command::Write(Write::Zrem(Zrem::parse_frames(parse_ref)?)),
            "zremrangebylex" => Command::Write(Write::Zremrangebylex(
                Zremrangebylex::parse_frames(parse_ref)?,
            )),
            "zremrangebyrank" => Command::Write(Write::Zremrangebyrank(
                Zremrangebyrank::parse_frames(parse_ref)?,
            )),
            "zremrangebyscore" => Command::Write(Write::Zremrangebyscore(
                Zremrangebyscore::parse_frames(parse_ref)?,
            )),
            "zrevrange" => Command::Read(Read::Zrevrange(Zrevrange::parse_frames(parse_ref)?)),
            "zrevrangebylex" => Command::Read(Read::Zrevrangebylex(Zrevrangebylex::parse_frames(
                parse_ref,
            )?)),
            "zrevrangebyscore" => Command::Read(Read::Zrevrangebyscore(
                Zrevrangebyscore::parse_frames(parse_ref)?,
            )),
            "zrevrank" => Command::Read(Read::Zrevrank(Zrevrank::parse_frames(parse_ref)?)),
            "zrange" => Command::Read(Read::Zrange(Zrange::parse_frames(parse_ref)?)),
            "zadd" => Command::Write(Write::Zadd(Zadd::parse_frames(parse_ref)?)),
            "sadd" => Command::Write(Write::Sadd(Sadd::parse_frames(parse_ref)?)),
            "sismember" => Command::Read(Read::Sismember(Sismember::parse_frames(parse_ref)?)),
            "smismember" => Command::Read(Read::Smismember(Smismember::parse_frames(parse_ref)?)),
            "smembers" => Command::Read(Read::Smembers(Smembers::parse_frames(parse_ref)?)),
            "srem" => Command::Write(Write::Srem(Srem::parse_frames(parse_ref)?)),
            "hincrby" => Command::Write(Write::Hincrby(Hincrby::parse_frames(parse_ref)?)),
            "hexists" => Command::Read(Read::Hexists(Hexists::parse_frames(parse_ref)?)),
            "hdel" => Command::Write(Write::Hdel(Hdel::parse_frames(parse_ref)?)),
            "hsetnx" => Command::Write(Write::Hsetnx(Hsetnx::parse_frames(parse_ref)?)),
            "hget" => Command::Read(Read::Hget(Hget::parse_frames(parse_ref)?)),
            "hmget" => Command::Read(Read::Hmget(Hmget::parse_frames(parse_ref)?)),
            "hset" => Command::Write(Write::Hset(Hset::parse_frames(parse_ref)?)),
            "hgetall" => Command::Read(Read::Hgetall(Hgetall::parse_frames(parse_ref)?)),
            "llen" => Command::Read(Read::Llen(Llen::parse_frames(parse_ref)?)),
            "rpop" => Command::Write(Write::Rpop(Rpop::parse_frames(parse_ref)?)),
            "lpop" => Command::Write(Write::Lpop(Lpop::parse_frames(parse_ref)?)),
            "lrange" => Command::Read(Read::Lrange(Lrange::parse_frames(parse_ref)?)),
            "lpush" => Command::Write(Write::Lpush(Lpush::parse_frames(parse_ref)?)),
            "rpush" => Command::Write(Write::Rpush(Rpush::parse_frames(parse_ref)?)),
            "lpushx" => Command::Write(Write::Lpushx(Lpushx::parse_frames(parse_ref)?)),
            "rpushx" => Command::Write(Write::Rpushx(Rpushx::parse_frames(parse_ref)?)),
            "incrby" => Command::Write(Write::Incrby(Incrby::parse_frames(parse_ref)?)),
            "incr" => Command::Write(Write::Incr(Incr::parse_frames(parse_ref)?)),
            "decrby" => Command::Write(Write::Decrby(Decrby::parse_frames(parse_ref)?)),
            "decr" => Command::Write(Write::Decr(Decr::parse_frames(parse_ref)?)),
            "get" => Command::Read(Read::Get(Get::parse_frames(parse_ref)?)),
            "set" => Command::Write(Write::Set(Set::parse_frames(parse_ref)?)),
            "del" => Command::Write(Write::Del(Del::parse_frames(parse_ref)?)),
            "exists" => Command::Read(Read::Exists(Exists::parse_frames(parse_ref)?)),
            "psetex" => Command::Write(Write::Psetex(Psetex::parse_frames(parse_ref)?)),
            "setex" => Command::Write(Write::Setex(Setex::parse_frames(parse_ref)?)),
            "pexpireat" => Command::Write(Write::Pexpireat(Pexpireat::parse_frames(parse_ref)?)),
            "expireat" => Command::Write(Write::Expireat(Expireat::parse_frames(parse_ref)?)),
            "expire" => Command::Write(Write::Expire(Expire::parse_frames(parse_ref)?)),
            "pexpire" => Command::Write(Write::Pexpire(Pexpire::parse_frames(parse_ref)?)),
            "syncsnapshot" => Command::SyncSnapshot(SyncSnapshot::parse_frames(parse_ref)?),
            "flushall" => Command::Write(Write::Flushall(Flushall::parse_frames(parse_ref)?)),
            "info" => Command::Read(Read::Info(Info)),
            "dump" => Command::Read(Read::Dump(Dump::parse_frames(parse_ref)?)),
            _ => {
                // The command is not recognized and an Unknown command is
                // returned.
                //
                // `return` is called here to skip the `finish()` call below. As
                // the command is not recognized, there is most likely
                // unconsumed fields remaining in the `Parse` instance.
                Command::Unknown(Unknown::new(&command_name, parse_ref))
            }
        };

        // Check if there is any remaining unconsumed fields in the `Parse`
        // value. If fields remain, this indicates an unexpected frame format
        // and an error is returned.
        parse.finish()?;

        // The command has been successfully parsed
        Ok(command)
    }
}

impl<'a> Write<'a> {
    #[inline]
    pub async fn apply(
        self,
        // connection: &'a mut Connection,
        db: &'a Arc<Db>,
    ) -> common::Result<Frame<'a>> {
        match self {
            Write::Set(cmd) => cmd.apply(db).await,
            Write::Psetex(cmd) => cmd.apply(db),
            Write::Setex(cmd) => cmd.apply(db),
            Write::Del(cmd) => cmd.apply(db).await,
            Write::Pexpireat(cmd) => cmd.apply(db),
            Write::Expireat(cmd) => cmd.apply(db),
            Write::Expire(cmd) => cmd.apply(db),
            Write::Pexpire(cmd) => cmd.apply(db),
            Write::Incrby(cmd) => cmd.apply(db),
            Write::Incr(cmd) => cmd.apply(db),
            Write::Decr(cmd) => cmd.apply(db),
            Write::Decrby(cmd) => cmd.apply(db),
            Write::Lpush(cmd) => cmd.apply(db),
            Write::Rpush(cmd) => cmd.apply(db),
            Write::Lpushx(cmd) => cmd.apply(db),
            Write::Rpushx(cmd) => cmd.apply(db),
            Write::Lpop(cmd) => cmd.apply(db),
            Write::Rpop(cmd) => cmd.apply(db),
            Write::Hset(cmd) => cmd.apply(db),
            Write::Hdel(cmd) => cmd.apply(db),
            Write::Hsetnx(cmd) => cmd.apply(db),
            Write::Hincrby(cmd) => cmd.apply(db),
            Write::Sadd(cmd) => cmd.apply(db),
            Write::Srem(cmd) => cmd.apply(db),
            Write::Zadd(cmd) => cmd.apply(db),
            Write::Zrem(cmd) => cmd.apply(db),
            Write::Zremrangebyrank(cmd) => cmd.apply(db),
            Write::Zremrangebyscore(cmd) => cmd.apply(db),
            Write::Zremrangebylex(cmd) => cmd.apply(db),
            Write::Flushall(cmd) => Ok(cmd.apply(db.clone())),
        }
    }
}

impl Read<'_> {
    #[inline]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        match self {
            Read::Get(cmd) => cmd.apply(db),
            Read::Llen(cmd) => cmd.apply(db),
            Read::Hgetall(cmd) => cmd.apply(db),
            Read::Hget(cmd) => cmd.apply(db),
            Read::Hmget(cmd) => cmd.apply(db),
            Read::Hexists(cmd) => cmd.apply(db),
            Read::Sismember(cmd) => cmd.apply(db),
            Read::Smembers(cmd) => cmd.apply(db),
            Read::Smismember(cmd) => cmd.apply(db),
            Read::Zrangebylex(cmd) => cmd.apply(db),
            Read::Zrangebyscore(cmd) => cmd.apply(db),
            Read::Zrank(cmd) => cmd.apply(db),
            Read::Zrevrange(cmd) => cmd.apply(db),
            Read::Zrevrangebylex(cmd) => cmd.apply(db),
            Read::Zrevrangebyscore(cmd) => cmd.apply(db),
            Read::Zrevrank(cmd) => cmd.apply(db),
            Read::Zrange(cmd) => cmd.apply(db),
            Read::Lrange(cmd) => cmd.apply(db),
            Read::Exists(cmd) => cmd.apply(db),
            Read::Ttl(cmd) => cmd.apply(db),
            Read::Pttl(cmd) => cmd.apply(db),
            Read::Info(cmd) => Ok(cmd.apply(db)),
            Read::Dump(cmd) => cmd.apply(db),
        }
    }
}
