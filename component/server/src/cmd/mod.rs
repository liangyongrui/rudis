mod base;
mod hash;
mod list;
mod set;
mod sorted_set;
mod syncsnapshot;

use connection::{
    parse::{frame::Frame, Parse},
    Connection,
};
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
pub enum Command {
    Ping,
    Read(Read),
    Write(Write),
    SyncSnapshot(SyncSnapshot),
    Unknown(Unknown),
}
#[derive(Debug)]
pub enum Read {
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
    Ttl(Ttl),
    Pttl(Pttl),
    Exists(Exists),
}

#[derive(Debug, Clone)]
pub enum Write {
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
    /// The `Frame` must represent a Redis command supported by `rudis` and
    /// be the array variant.
    ///
    /// # Returns
    ///
    /// On success, the command value is returned, otherwise, `Err` is returned.
    pub fn from_frame(frame: Frame) -> common::Result<Command> {
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
            "ping" => Command::Ping,
            "ttl" => Command::Read(Read::Ttl(Ttl::parse_frames(&mut parse)?)),
            "pttl" => Command::Read(Read::Pttl(Pttl::parse_frames(&mut parse)?)),
            "zrangebylex" => {
                Command::Read(Read::Zrangebylex(Zrangebylex::parse_frames(&mut parse)?))
            }
            "zrangebyscore" => Command::Read(Read::Zrangebyscore(Zrangebyscore::parse_frames(
                &mut parse,
            )?)),
            "zrank" => Command::Read(Read::Zrank(Zrank::parse_frames(&mut parse)?)),
            "zrem" => Command::Write(Write::Zrem(Zrem::parse_frames(&mut parse)?)),
            "zremrangebylex" => Command::Write(Write::Zremrangebylex(
                Zremrangebylex::parse_frames(&mut parse)?,
            )),
            "zremrangebyrank" => Command::Write(Write::Zremrangebyrank(
                Zremrangebyrank::parse_frames(&mut parse)?,
            )),
            "zremrangebyscore" => Command::Write(Write::Zremrangebyscore(
                Zremrangebyscore::parse_frames(&mut parse)?,
            )),
            "zrevrange" => Command::Read(Read::Zrevrange(Zrevrange::parse_frames(&mut parse)?)),
            "zrevrangebylex" => Command::Read(Read::Zrevrangebylex(Zrevrangebylex::parse_frames(
                &mut parse,
            )?)),
            "zrevrangebyscore" => Command::Read(Read::Zrevrangebyscore(
                Zrevrangebyscore::parse_frames(&mut parse)?,
            )),
            "zrevrank" => Command::Read(Read::Zrevrank(Zrevrank::parse_frames(&mut parse)?)),
            "zrange" => Command::Read(Read::Zrange(Zrange::parse_frames(&mut parse)?)),
            "zadd" => Command::Write(Write::Zadd(Zadd::parse_frames(&mut parse)?)),
            "sadd" => Command::Write(Write::Sadd(Sadd::parse_frames(&mut parse)?)),
            "sismember" => Command::Read(Read::Sismember(Sismember::parse_frames(&mut parse)?)),
            "smismember" => Command::Read(Read::Smismember(Smismember::parse_frames(&mut parse)?)),
            "smembers" => Command::Read(Read::Smembers(Smembers::parse_frames(&mut parse)?)),
            "srem" => Command::Write(Write::Srem(Srem::parse_frames(&mut parse)?)),
            "hincrby" => Command::Write(Write::Hincrby(Hincrby::parse_frames(&mut parse)?)),
            "hexists" => Command::Read(Read::Hexists(Hexists::parse_frames(&mut parse)?)),
            "hdel" => Command::Write(Write::Hdel(Hdel::parse_frames(&mut parse)?)),
            "hsetnx" => Command::Write(Write::Hsetnx(Hsetnx::parse_frames(&mut parse)?)),
            "hget" => Command::Read(Read::Hget(Hget::parse_frames(&mut parse)?)),
            "hmget" => Command::Read(Read::Hmget(Hmget::parse_frames(&mut parse)?)),
            "hset" => Command::Write(Write::Hset(Hset::parse_frames(&mut parse)?)),
            "hgetall" => Command::Read(Read::Hgetall(Hgetall::parse_frames(&mut parse)?)),
            "llen" => Command::Read(Read::Llen(Llen::parse_frames(&mut parse)?)),
            "rpop" => Command::Write(Write::Rpop(Rpop::parse_frames(&mut parse)?)),
            "lpop" => Command::Write(Write::Lpop(Lpop::parse_frames(&mut parse)?)),
            "lrange" => Command::Read(Read::Lrange(Lrange::parse_frames(&mut parse)?)),
            "lpush" => Command::Write(Write::Lpush(Lpush::parse_frames(&mut parse)?)),
            "rpush" => Command::Write(Write::Rpush(Rpush::parse_frames(&mut parse)?)),
            "lpushx" => Command::Write(Write::Lpushx(Lpushx::parse_frames(&mut parse)?)),
            "rpushx" => Command::Write(Write::Rpushx(Rpushx::parse_frames(&mut parse)?)),
            "incrby" => Command::Write(Write::Incrby(Incrby::parse_frames(&mut parse)?)),
            "incr" => Command::Write(Write::Incr(Incr::parse_frames(&mut parse)?)),
            "decrby" => Command::Write(Write::Decrby(Decrby::parse_frames(&mut parse)?)),
            "decr" => Command::Write(Write::Decr(Decr::parse_frames(&mut parse)?)),
            "get" => Command::Read(Read::Get(Get::parse_frames(&mut parse)?)),
            "set" => Command::Write(Write::Set(Set::parse_frames(&mut parse)?)),
            "del" => Command::Write(Write::Del(Del::parse_frames(&mut parse)?)),
            "exists" => Command::Read(Read::Exists(Exists::parse_frames(&mut parse)?)),
            "psetex" => Command::Write(Write::Psetex(Psetex::parse_frames(&mut parse)?)),
            "setex" => Command::Write(Write::Setex(Setex::parse_frames(&mut parse)?)),
            "pexpireat" => Command::Write(Write::Pexpireat(Pexpireat::parse_frames(&mut parse)?)),
            "expireat" => Command::Write(Write::Expireat(Expireat::parse_frames(&mut parse)?)),
            "expire" => Command::Write(Write::Expire(Expire::parse_frames(&mut parse)?)),
            "pexpire" => Command::Write(Write::Pexpire(Pexpire::parse_frames(&mut parse)?)),
            "syncsnapshot" => Command::SyncSnapshot(SyncSnapshot::parse_frames(&mut parse)?),
            _ => {
                // The command is not recognized and an Unknown command is
                // returned.
                //
                // `return` is called here to skip the `finish()` call below. As
                // the command is not recognized, there is most likely
                // unconsumed fields remaining in the `Parse` instance.
                return Ok(Command::Unknown(Unknown::new(&command_name, &mut parse)));
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

impl Write {
    #[inline]
    pub async fn apply(self, connection: &mut Connection, db: &Db) -> common::Result<Frame> {
        use Write::{
            Decr, Decrby, Del, Expire, Expireat, Hdel, Hincrby, Hset, Hsetnx, Incr, Incrby, Lpop,
            Lpush, Lpushx, Pexpire, Pexpireat, Psetex, Rpop, Rpush, Rpushx, Sadd, Set, Setex, Srem,
            Zadd, Zrem, Zremrangebylex, Zremrangebyrank, Zremrangebyscore,
        };
        match self {
            Set(cmd) => cmd.apply(connection, db).await,
            Psetex(cmd) => cmd.apply(db),
            Setex(cmd) => cmd.apply(db),
            Del(cmd) => cmd.apply(connection, db).await,
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

impl Read {
    #[inline]
    pub fn apply(self, db: &Db) -> common::Result<Frame> {
        use Read::{
            Exists, Get, Hexists, Hget, Hgetall, Hmget, Llen, Lrange, Pttl, Sismember, Smembers,
            Smismember, Ttl, Zrange, Zrangebylex, Zrangebyscore, Zrank, Zrevrange, Zrevrangebylex,
            Zrevrangebyscore, Zrevrank,
        };

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
            Ttl(cmd) => cmd.apply(db),
            Pttl(cmd) => cmd.apply(db),
        }
    }
}
