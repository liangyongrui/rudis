use std::ops::Deref;

use super::Data;

#[derive(Debug, Clone)]
pub struct Bytes(bytes::Bytes);

impl Deref for Bytes {
    type Target = bytes::Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Bytes {
    pub fn get_inner_clone(&self) -> bytes::Bytes {
        self.0.clone()
    }
    pub fn get_inner(self) -> bytes::Bytes {
        self.0
    }
}

impl From<bytes::Bytes> for Data {
    fn from(bytes: bytes::Bytes) -> Self {
        Self::Bytes(Bytes(bytes))
    }
}

impl From<bytes::Bytes> for Bytes {
    fn from(bytes: bytes::Bytes) -> Self {
        Bytes(bytes)
    }
}
