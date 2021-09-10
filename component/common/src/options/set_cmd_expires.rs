use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum SetCmdExpires {
    Ex(u64),
    Px(u64),
    Exat(u64),
    Pxat(u64),
    Keepttl,
    None,
}

impl Default for SetCmdExpires {
    fn default() -> Self {
        Self::None
    }
}

impl SetCmdExpires {
    /// # Errors
    /// no errors
    #[inline]
    pub fn parse_frames(
        tag: &str,
        parse: &crate::connection::parse::Parse,
    ) -> crate::Result<Option<Self>> {
        let res = match tag {
            "ex" => Some(Self::Ex(parse.next_int()? as _)),
            "px" => Some(Self::Px(parse.next_int()? as _)),
            "exat" => Some(Self::Exat(parse.next_int()? as _)),
            "pxat" => Some(Self::Pxat(parse.next_int()? as _)),
            "keepttl" => Some(Self::Keepttl),
            _ => None,
        };
        Ok(res)
    }
}
