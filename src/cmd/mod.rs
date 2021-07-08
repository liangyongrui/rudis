mod base;
mod hash;
mod list;
mod set;
mod sorted_set;

pub use self::base::{get::Get, set::Set, setex::Setex};
use self::{
    base::{
        decr::Decr, decrby::Decrby, del::Del, exists::Exists, expire::Expire, expireat::Expireat,
        incr::Incr, incrby::Incrby, pexpire::Pexpire, pexpireat::Pexpireat, psetex::Psetex,
        unknown::Unknown,
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
    Zrangebylex(Zrangebylex),
    Zrangebyscore(Zrangebyscore),
    Zrank(Zrank),
    Zrem(Zrem),
    Zremrangebyrank(Zremrangebyrank),
    Zremrangebyscore(Zremrangebyscore),
    Zrevrange(Zrevrange),
    Zrevrangebylex(Zrevrangebylex),
    Zrevrangebyscore(Zrevrangebyscore),
    Zrevrank(Zrevrank),
    Zrange(Zrange),
    Zadd(Zadd),
    Sadd(Sadd),
    Sismember(Sismember),
    Smembers(Smembers),
    Smismember(Smismember),
    Srem(Srem),
    Hincrby(Hincrby),
    Hexists(Hexists),
    Hdel(Hdel),
    Hsetnx(Hsetnx),
    Hget(Hget),
    Hmget(Hmget),
    Hset(Hset),
    Hgetall(Hgetall),
    Lpop(Lpop),
    Llen(Llen),
    Rpop(Rpop),
    Lrange(Lrange),
    Lpush(Lpush),
    Rpush(Rpush),
    Lpushx(Lpushx),
    Rpushx(Rpushx),
    Incrby(Incrby),
    Incr(Incr),
    Decr(Decr),
    Decrby(Decrby),
    Get(Get),
    Set(Set),
    Del(Del),
    Exists(Exists),
    Psetex(Psetex),
    Setex(Setex),
    Pexpireat(Pexpireat),
    Expireat(Expireat),
    Expire(Expire),
    Pexpire(Pexpire),
    Unknown(Unknown),
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
            "zrangebylex" => Command::Zrangebylex(Zrangebylex::parse_frames(&mut parse)?),
            "zrangebyscore" => Command::Zrangebyscore(Zrangebyscore::parse_frames(&mut parse)?),
            "zrank" => Command::Zrank(Zrank::parse_frames(&mut parse)?),
            "zrem" => Command::Zrem(Zrem::parse_frames(&mut parse)?),
            "zremrangebyrank" => {
                Command::Zremrangebyrank(Zremrangebyrank::parse_frames(&mut parse)?)
            }
            "zremrangebyscore" => {
                Command::Zremrangebyscore(Zremrangebyscore::parse_frames(&mut parse)?)
            }
            "zrevrange" => Command::Zrevrange(Zrevrange::parse_frames(&mut parse)?),
            "zrevrangebylex" => Command::Zrevrangebylex(Zrevrangebylex::parse_frames(&mut parse)?),
            "zrevrangebyscore" => {
                Command::Zrevrangebyscore(Zrevrangebyscore::parse_frames(&mut parse)?)
            }
            "zrevrank" => Command::Zrevrank(Zrevrank::parse_frames(&mut parse)?),
            "zrange" => Command::Zrange(Zrange::parse_frames(&mut parse)?),
            "zadd" => Command::Zadd(Zadd::parse_frames(&mut parse)?),
            "sadd" => Command::Sadd(Sadd::parse_frames(&mut parse)?),
            "sismember" => Command::Sismember(Sismember::parse_frames(&mut parse)?),
            "smismember" => Command::Smismember(Smismember::parse_frames(&mut parse)?),
            "smembers" => Command::Smembers(Smembers::parse_frames(&mut parse)?),
            "srem" => Command::Srem(Srem::parse_frames(&mut parse)?),
            "hincrby" => Command::Hincrby(Hincrby::parse_frames(&mut parse)?),
            "hexist" => Command::Hexists(Hexists::parse_frames(&mut parse)?),
            "hdel" => Command::Hdel(Hdel::parse_frames(&mut parse)?),
            "hsetnx" => Command::Hsetnx(Hsetnx::parse_frames(&mut parse)?),
            "hget" => Command::Hget(Hget::parse_frames(&mut parse)?),
            "hmget" => Command::Hmget(Hmget::parse_frames(&mut parse)?),
            "hset" => Command::Hset(Hset::parse_frames(&mut parse)?),
            "hgetall" => Command::Hgetall(Hgetall::parse_frames(&mut parse)?),
            "llen" => Command::Llen(Llen::parse_frames(&mut parse)?),
            "rpop" => Command::Rpop(Rpop::parse_frames(&mut parse)?),
            "lpop" => Command::Lpop(Lpop::parse_frames(&mut parse)?),
            "lrange" => Command::Lrange(Lrange::parse_frames(&mut parse)?),
            "lpush" => Command::Lpush(Lpush::parse_frames(&mut parse)?),
            "rpush" => Command::Rpush(Rpush::parse_frames(&mut parse)?),
            "lpushx" => Command::Lpushx(Lpushx::parse_frames(&mut parse)?),
            "rpushx" => Command::Rpushx(Rpushx::parse_frames(&mut parse)?),
            "incrby" => Command::Incrby(Incrby::parse_frames(&mut parse)?),
            "incr" => Command::Incr(Incr::parse_frames(&mut parse)?),
            "decrby" => Command::Decrby(Decrby::parse_frames(&mut parse)?),
            "decr" => Command::Decr(Decr::parse_frames(&mut parse)?),
            "get" => Command::Get(Get::parse_frames(&mut parse)?),
            "set" => Command::Set(Set::parse_frames(&mut parse)?),
            "del" => Command::Del(Del::parse_frames(&mut parse)?),
            "exists" => Command::Exists(Exists::parse_frames(&mut parse)?),
            "psetex" => Command::Psetex(Psetex::parse_frames(&mut parse)?),
            "setex" => Command::Setex(Setex::parse_frames(&mut parse)?),
            "pexpireat" => Command::Pexpireat(Pexpireat::parse_frames(&mut parse)?),
            "expireat" => Command::Expireat(Expireat::parse_frames(&mut parse)?),
            "expire" => Command::Expire(Expire::parse_frames(&mut parse)?),
            "pexpire" => Command::Pexpire(Pexpire::parse_frames(&mut parse)?),
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
    pub(crate) async fn apply(
        self,
        db: &Db,
        dst: &mut Connection,
        _shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        use Command::*;

        match self {
            Get(cmd) => cmd.apply(db, dst).await,
            Set(cmd) => cmd.apply(db, dst).await,
            Unknown(cmd) => cmd.apply(dst).await,
            Psetex(cmd) => cmd.apply(db, dst).await,
            Setex(cmd) => cmd.apply(db, dst).await,
            Del(cmd) => cmd.apply(db, dst).await,
            Exists(cmd) => cmd.apply(db, dst).await,
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
            Lrange(cmd) => cmd.apply(db, dst).await,
            Lpop(cmd) => cmd.apply(db, dst).await,
            Llen(cmd) => cmd.apply(db, dst).await,
            Rpop(cmd) => cmd.apply(db, dst).await,
            Hset(cmd) => cmd.apply(db, dst).await,
            Hgetall(cmd) => cmd.apply(db, dst).await,
            Hget(cmd) => cmd.apply(db, dst).await,
            Hmget(cmd) => cmd.apply(db, dst).await,
            Hdel(cmd) => cmd.apply(db, dst).await,
            Hsetnx(cmd) => cmd.apply(db, dst).await,
            Hincrby(cmd) => cmd.apply(db, dst).await,
            Hexists(cmd) => cmd.apply(db, dst).await,
            Sadd(cmd) => cmd.apply(db, dst).await,
            Sismember(cmd) => cmd.apply(db, dst).await,
            Smembers(cmd) => cmd.apply(db, dst).await,
            Smismember(cmd) => cmd.apply(db, dst).await,
            Srem(cmd) => cmd.apply(db, dst).await,
            Zrange(cmd) => cmd.apply(db, dst).await,
            Zadd(cmd) => cmd.apply(db, dst).await,
            Zrangebylex(cmd) => cmd.apply(db, dst).await,
            Zrangebyscore(cmd) => cmd.apply(db, dst).await,
            Zrank(cmd) => cmd.apply(db, dst).await,
            Zrem(cmd) => cmd.apply(db, dst).await,
            Zremrangebyrank(cmd) => cmd.apply(db, dst).await,
            Zremrangebyscore(cmd) => cmd.apply(db, dst).await,
            Zrevrange(cmd) => cmd.apply(db, dst).await,
            Zrevrangebylex(cmd) => cmd.apply(db, dst).await,
            Zrevrangebyscore(cmd) => cmd.apply(db, dst).await,
            Zrevrank(cmd) => cmd.apply(db, dst).await,
        }
    }
}
