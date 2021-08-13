use std::sync::Arc;

use crate::{
    cmd::{Parse, ParseError},
    db::Db,
    Frame,
};
use common::{
    now_timestamp_ms,
    options::{ExpiresAt, NxXx},
};
use dict::data_type::DataType;

/// Set `key` to hold the string `value`.
///
/// https://redis.io/commands/set
#[derive(Debug, Clone)]
pub struct Set {
    /// the lookup key
    pub key: Arc<[u8]>,
    /// the value to be stored
    pub value: DataType,
    // None not set, true nx, false xx
    pub nx_xx: NxXx,
    /// When to expire the key
    ///
    /// unix timestatmp ms
    pub expires_at: u64,
    pub keepttl: bool,
    pub get: bool,
}

impl From<Set> for dict::cmd::simple::set::Req {
    fn from(old: Set) -> Self {
        Self {
            key: old.key,
            value: old.value,
            expires_at: if old.keepttl {
                ExpiresAt::Last
            } else {
                ExpiresAt::Specific(old.expires_at)
            },
            nx_xx: old.nx_xx,
        }
    }
}

impl Set {
    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Set> {
        use ParseError::EndOfStream;

        // Read the key to set. This is a required field
        let key = parse.next_key()?;

        // Read the value to set. This is a required field.
        let value = parse.next_data()?;

        // The expiration is optional. If nothing else follows, then it is
        // `None`.
        let mut expires_at = 0;
        let mut nx_xx = NxXx::None;
        let mut keepttl = false;
        let mut get = false;
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
                    "EX" => {
                        if expires_at > 0 {
                            return Err("expiration already set".into());
                        }
                        let secs = parse.next_int()?;
                        expires_at = now_timestamp_ms() + secs as u64 * 1000;
                    }
                    "PX" => {
                        if expires_at > 0 {
                            return Err("expiration already set".into());
                        }
                        let ms = parse.next_int()?;
                        expires_at = now_timestamp_ms() + ms as u64;
                    }
                    "EXAT" => {
                        if expires_at > 0 {
                            return Err("expiration already set".into());
                        }
                        let secs_timestamp = parse.next_int()?;
                        expires_at = secs_timestamp as u64 * 1000;
                    }
                    "PXAT" => {
                        if expires_at > 0 {
                            return Err("expiration already set".into());
                        }
                        let ms_timestamp = parse.next_int()?;
                        expires_at = ms_timestamp as _;
                    }
                    "KEEPTTL" => {
                        keepttl = true;
                    }
                    "GET" => {
                        get = true;
                    }
                    not_support => return Err(format!("not support cmd: {}", not_support).into()),
                },
                Err(EndOfStream) => {
                    break;
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(Set {
            key,
            value,
            nx_xx,
            expires_at,
            keepttl,
            get,
        })
    }

    #[tracing::instrument(skip(self, db), level = "debug")]
    pub fn apply(self, db: &Db) -> crate::Result<Frame> {
        let get = self.get;
        let res = db.set(self.into())?;
        let response = if get { res.into() } else { Frame::ok() };
        Ok(response)
    }
}
