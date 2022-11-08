use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum Freq {
    Some(u64),
    None,
}

impl Default for Freq {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl Freq {
    /// # Errors
    /// parse error
    #[inline]
    pub fn parse_frames(
        tag: &str,
        parse: &crate::connection::parse::Parse,
    ) -> crate::Result<Option<Self>> {
        let res = match tag {
            "freq" => Some(Self::Some(parse.next_int()? as _)),
            _ => None,
        };
        Ok(res)
    }
}
