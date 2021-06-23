mod get;
pub use get::Get;

mod publish;
pub use publish::Publish;

mod set;
pub use set::Set;

mod subscribe;
pub use subscribe::{Subscribe, Unsubscribe};

mod unknown;
pub use unknown::Unknown;

mod psetex;
pub use psetex::Psetex;

mod setex;
pub use setex::Setex;

mod del;
pub use del::Del;

mod exists;
pub use exists::Exists;

mod pexpireat;
pub use pexpireat::Pexpireat;

use crate::{Connection, Db, Frame, Parse, ParseError, Shutdown};

/// Enumeration of supported Redis commands.
///
/// Methods called on `Command` are delegated to the command implementation.
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    Del(Del),
    Exists(Exists),
    Psetex(Psetex),
    Setex(Setex),
    Pexpireat(Pexpireat),
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
            "get" => Command::Get(Get::parse_frames(&mut parse)?),
            "set" => Command::Set(Set::parse_frames(&mut parse)?),
            "del" => Command::Del(Del::parse_frames(&mut parse)?),
            "exists" => Command::Exists(Exists::parse_frames(&mut parse)?),
            "psetex" => Command::Psetex(Psetex::parse_frames(&mut parse)?),
            "setex" => Command::Setex(Setex::parse_frames(&mut parse)?),
            "pexpireat" =>Command::Pexpireat(Pexpireat::parse_frames(&mut parse)?),
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
        }
    }

    /// Returns the command name
    pub(crate) fn get_name(&self) -> &str {
        match self {
            Command::Get(_) => "get",
            Command::Publish(_) => "pub",
            Command::Set(_) => "set",
            Command::Subscribe(_) => "subscribe",
            Command::Unsubscribe(_) => "unsubscribe",
            Command::Unknown(cmd) => cmd.get_name(),
            Command::Psetex(_) => "psetex",
            Command::Setex(_) => "setex",
            Command::Del(_) => "del",
            Command::Exists(_) => "exists",
            Command::Pexpireat(_) => "pexpireat",
        }
    }
}
