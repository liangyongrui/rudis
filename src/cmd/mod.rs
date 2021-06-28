mod decr;
mod decrby;
mod del;
mod exists;
mod expire;
mod expireat;
mod get;
mod hash;
mod incr;
mod incrby;
mod list;
mod pexpire;
mod pexpireat;
mod psetex;
mod publish;
mod set;
mod setex;
mod subscribe;
mod unknown;

pub use decr::Decr;
pub use decrby::Decrby;
pub use del::Del;
pub use exists::Exists;
pub use expire::Expire;
pub use expireat::Expireat;
pub use get::Get;
pub use incr::Incr;
pub use incrby::Incrby;
pub use pexpire::Pexpire;
pub use pexpireat::Pexpireat;
pub use psetex::Psetex;
pub use publish::Publish;
pub use set::Set;
pub use setex::Setex;
pub use subscribe::{Subscribe, Unsubscribe};
pub use unknown::Unknown;

use self::{
    hash::{hget::Hget, hgetall::Hgetall, hmget::Hmget, hset::Hset},
    list::{
        llen::Llen, lpop::Lpop, lpush::Lpush, lpushx::Lpushx, lrange::Lrange, rpop::Rpop,
        rpush::Rpush, rpushx::Rpushx,
    },
};
use crate::{Connection, Db, Frame, Parse, ParseError, Shutdown};

/// Enumeration of supported Redis commands.
///
/// Methods called on `Command` are delegated to the command implementation.
#[derive(Debug)]
pub enum Command {
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
    Publish(Publish),
    Subscribe(Subscribe),
    Unsubscribe(Unsubscribe),
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
            "publish" => Command::Publish(Publish::parse_frames(&mut parse)?),
            "subscribe" => Command::Subscribe(Subscribe::parse_frames(&mut parse)?),
            "unsubscribe" => Command::Unsubscribe(Unsubscribe::parse_frames(&mut parse)?),
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
        shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        use Command::*;

        match self {
            // `Unsubscribe` cannot be applied. It may only be received from the
            // context of a `Subscribe` command.
            Unsubscribe(_) => Err("`Unsubscribe` is unsupported in this context".into()),
            Get(cmd) => cmd.apply(db, dst).await,
            Publish(cmd) => cmd.apply(db, dst).await,
            Set(cmd) => cmd.apply(db, dst).await,
            Subscribe(cmd) => cmd.apply(db, dst, shutdown).await,
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
        }
    }

    /// Returns the command name
    pub(crate) fn get_name(&self) -> &str {
        match self {
            Command::Unknown(cmd) => cmd.get_name(),
            Command::Get(_) => "get",
            Command::Publish(_) => "pub",
            Command::Set(_) => "set",
            Command::Subscribe(_) => "subscribe",
            Command::Unsubscribe(_) => "unsubscribe",
            Command::Psetex(_) => "psetex",
            Command::Setex(_) => "setex",
            Command::Del(_) => "del",
            Command::Exists(_) => "exists",
            Command::Pexpireat(_) => "pexpireat",
            Command::Expireat(_) => "expireat",
            Command::Expire(_) => "expire",
            Command::Pexpire(_) => "pexpire",
            Command::Incrby(_) => "incrby",
            Command::Incr(_) => "incr",
            Command::Decr(_) => "decr",
            Command::Decrby(_) => "decrby",
            Command::Lpush(_) => "lpush",
            Command::Rpush(_) => "rpush",
            Command::Lpushx(_) => "lpushx",
            Command::Rpushx(_) => "rpushx",
            Command::Lrange(_) => "lrange",
            Command::Lpop(_) => "lpop",
            Command::Llen(_) => "llen",
            Command::Rpop(_) => "rpop",
            Command::Hset(_) => "hset",
            Command::Hgetall(_) => "hgetall",
            Command::Hget(_) => "hget",
            Command::Hmget(_) => "hmget",
        }
    }
}
