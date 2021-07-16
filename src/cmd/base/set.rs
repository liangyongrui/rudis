use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use tracing::{debug, instrument};

use crate::{
    cmd::{Parse, ParseError},
    db::data_type::SimpleType,
    utils::options::NxXx,
    Connection, Db, Frame,
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
    #[instrument(skip(self, db, dst))]
    pub async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let res = db
            .set(
                self.key,
                self.value,
                self.nx_xx,
                self.expires_at,
                self.keepttl,
            )
            .await;
        let response = if self.get {
            match res {
                Ok(Some(SimpleType::Blob(value))) => Frame::Bulk(value.into()),
                Ok(Some(SimpleType::SimpleString(value))) => Frame::Simple(value),
                Ok(Some(SimpleType::Integer(value))) => Frame::Integer(value),
                Ok(Some(SimpleType::Null)) => Frame::Null,
                Ok(None) => Frame::Null,
                Err(e) => Frame::Error(e),
            }
        } else {
            Frame::Simple("OK".to_string())
        };
        debug!(?response);
        dst.write_frame(&response).await?;

        Ok(())
    }
}
