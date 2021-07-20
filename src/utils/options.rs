use bytes::Bytes;

#[derive(Debug, Clone, Copy)]
pub enum NxXx {
    Nx,
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
}

#[derive(Debug, Clone, Copy)]
pub enum GtLt {
    Gt,
    Lt,
    None,
}
