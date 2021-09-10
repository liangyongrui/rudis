use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum GtLt {
    Gt,
    Lt,
    None,
}

impl Default for GtLt {
    fn default() -> Self {
        Self::None
    }
}

impl GtLt {
    /// # Errors
    /// no errors
    #[inline]
    pub fn parse_frames(
        tag: &str,
        _parse: &crate::connection::parse::Parse,
    ) -> crate::Result<Option<Self>> {
        let res = match tag {
            "gt" => Some(GtLt::Gt),
            "lt" => Some(GtLt::Lt),
            _ => None,
        };
        Ok(res)
    }
    #[inline]
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, GtLt::None)
    }
}
