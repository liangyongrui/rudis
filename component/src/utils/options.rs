use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum NxXx {
    // Only set the key if it does not already exist.
    Nx,
    // Only set the key if it already exist.
    Xx,
    None,
}

impl From<NxXx> for Option<Bytes> {
    fn from(nx_xx: NxXx) -> Self {
        match nx_xx {
            NxXx::Nx => Some("NX".into()),
            NxXx::Xx => Some("XX".into()),
            NxXx::None => None,
        }
    }
}

impl NxXx {
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

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum GtLt {
    Gt,
    Lt,
    None,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ExpiresAt {
    // 指定时间, 0 不会过期
    Specific(u64),
    // 上一次时间
    Last,
}
