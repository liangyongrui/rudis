use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use tracing::instrument;

use crate::{
    cmd::{Parse, ParseError},
    db2::Db,
    slot::data_type::SimpleType,
    utils::options::{ExpiresAt, NxXx},
    Frame,
};

/// Set `key` to hold the string `value`.
///
/// https://redis.io/commands/set
#[derive(Debug, Clone)]
pub struct Set {
    /// the lookup key
    pub key: SimpleType,
    /// the value to be stored
    pub value: SimpleType,
    // None not set, true nx, false xx
    pub nx_xx: NxXx,
    /// When to expire the key
    ///
    /// unix timestatmp ms
    pub expires_at: Option<DateTime<Utc>>,
    pub keepttl: bool,
    pub get: bool,
}

impl From<Set> for crate::slot::cmd::simple::set::Req {
    fn from(old: Set) -> Self {
        Self {
            key: old.key,
            value: old.value,
            expires_at: if old.keepttl {
                ExpiresAt::Last
            } else {
                old.expires_at.into()
            },
            nx_xx: old.nx_xx,
        }
    }
}

impl Set {
    /// Create a new `Set` command which sets `key` to `value`.
    pub fn new(
        key: SimpleType,
        value: SimpleType,
        nx_xx: NxXx,
        expires_at: Option<DateTime<Utc>>,
        keepttl: bool,
        get: bool,
    ) -> Set {
        Set {
            key,
            value,
            nx_xx,
            expires_at,
            keepttl,
            get,
        }
    }

    /// Parse a `Set` instance from a received frame.
    ///
    /// The `Parse` argument provides a cursor-like API to read fields from the
    /// `Frame`. At this point, the entire frame has already been received from
    /// the socket.
    ///
    /// The `SET` string has already been consumed.
    ///
    /// # Returns
    ///
    /// Returns the `Set` value on success. If the frame is malformed, `Err` is
    /// returned.
    ///
    /// # Format
    ///
    /// Expects an array frame containing at least 3 entries.
    ///
    /// ```text
    /// SET key value [EX seconds|PX milliseconds]
    /// ```
    pub fn parse_frames(parse: &mut Parse) -> crate::Result<Set> {
        use ParseError::EndOfStream;

        // Read the key to set. This is a required field
        let key = parse.next_simple_type()?;

        // Read the value to set. This is a required field.
        let value = parse.next_simple_type()?;

        // The expiration is optional. If nothing else follows, then it is
        // `None`.
        let mut expires_at = None;
        let mut nx_xx = NxXx::None;
        let mut keepttl = false;
        let mut get = false;
        loop {
            // Attempt to parse another string.
            match parse.next_string() {
                Ok(s) => {
                    match &s.to_uppercase()[..] {
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
                            if expires_at.is_some() {
                                return Err("expiration already set".into());
                            }
                            // An expiration is specified in seconds. The next value is an
                            // integer.
                            let secs = parse.next_int()?;
                            expires_at = Utc::now().checked_add_signed(Duration::seconds(secs));
                        }
                        "PX" => {
                            if expires_at.is_some() {
                                return Err("expiration already set".into());
                            }
                            // An expiration is specified in milliseconds. The next value is
                            // an integer.
                            let ms = parse.next_int()?;
                            expires_at = Utc::now().checked_add_signed(Duration::milliseconds(ms));
                        }
                        "EXAT" => {
                            if expires_at.is_some() {
                                return Err("expiration already set".into());
                            }
                            let secs_timestamp = parse.next_int()?;
                            expires_at = Some(DateTime::<Utc>::from_utc(
                                NaiveDateTime::from_timestamp(secs_timestamp, 0),
                                Utc,
                            ));
                        }
                        "PXAT" => {
                            if expires_at.is_some() {
                                return Err("expiration already set".into());
                            }
                            let ms_timestamp = parse.next_int()?;
                            expires_at = Some(DateTime::<Utc>::from_utc(
                                NaiveDateTime::from_timestamp(ms_timestamp / 1000, 0),
                                Utc,
                            ));
                        }
                        "KEEPTTL" => {
                            keepttl = true;
                        }
                        "GET" => {
                            get = true;
                        }
                        not_support => {
                            return Err(format!("not support cmd: {}", not_support).into())
                        }
                    }
                }
                Err(EndOfStream) => {
                    break;
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(Self::new(key, value, nx_xx, expires_at, keepttl, get))
    }

    /// Apply the `Set` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, db))]
    pub async fn apply(self, db: &Db) -> crate::Result<Frame> {
        let get = self.get;
        let res = db.set(self.into()).await?;
        let response = if get {
            (&res).into()
        } else {
            Frame::Simple("OK".to_string())
        };
        Ok(response)
    }

    pub fn into_cmd_bytes(self) -> Vec<u8> {
        // let mut res = vec![
        //     Frame::Simple("SET".to_owned()),
        //     self.key.into(),
        //     self.value.into(),
        // ];
        // if let Some(ea) = self.expires_at {
        //     res.push(Frame::Simple("PXAT".to_owned()));
        //     res.push(Frame::Integer(ea.timestamp_millis()))
        // }
        // if self.keepttl {
        //     res.push(Frame::Simple("KEEPTTL".to_owned()));
        // }
        // match self.nx_xx {
        //     NxXx::Nx => res.push(Frame::Simple("NX".to_owned())),
        //     NxXx::Xx => res.push(Frame::Simple("XX".to_owned())),
        //     NxXx::None => (),
        // }
        // if self.get {
        //     res.push(Frame::Simple("GET".to_owned()))
        // }
        // Frame::Array(res).into()
        todo!()
    }
}
