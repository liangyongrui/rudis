use serde::{Deserialize, Serialize};

use crate::connection::parse::Parse;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum NxXx {
    // Only set the key if it does not already exist.
    Nx,
    // Only set the key if it already exist.
    Xx,
    None,
}

impl NxXx {
    /// # Errors
    /// no errors
    #[inline]
    pub fn parse_frames(tag: &str, _parse: &Parse) -> crate::Result<Option<Self>> {
        let res = match tag {
            "nx" => Some(NxXx::Nx),
            "xx" => Some(NxXx::Xx),
            _ => None,
        };
        Ok(res)
    }

    #[inline]
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, NxXx::None)
    }

    #[inline]
    #[must_use]
    pub const fn is_xx(self) -> bool {
        matches!(self, NxXx::Xx)
    }

    #[inline]
    #[must_use]
    pub const fn is_nx(self) -> bool {
        matches!(self, NxXx::Nx)
    }
}

impl Default for NxXx {
    fn default() -> Self {
        Self::None
    }
}
