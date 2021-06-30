use bytes::Bytes;

#[derive(Debug, PartialEq, Eq)]
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
    pub fn is_none(&self) -> bool {
        *self == NxXx::None
    }
}

#[derive(Debug)]
pub enum GtLt {
    Gt,
    Lt,
    None,
}
