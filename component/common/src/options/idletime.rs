use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum IdleTime {
    Some(u64),
    None,
}

impl Default for IdleTime {
    fn default() -> Self {
        Self::None
    }
}

impl IdleTime {
    /// # Errors
    /// no errors
    #[inline]
    pub fn parse_frames(
        tag: &str,
        parse: &crate::connection::parse::Parse,
    ) -> crate::Result<Option<Self>> {
        let res = match tag {
            "idletime" => Some(Self::Some(parse.next_int()? as _)),
            _ => None,
        };
        Ok(res)
    }
}
