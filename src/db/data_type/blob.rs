use std::ops::Deref;

use super::{DataType, SimpleType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Blob(bytes::Bytes);

impl Deref for Blob {
    type Target = bytes::Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Blob {
    pub fn get_inner(self) -> bytes::Bytes {
        self.0
    }
}

impl From<bytes::Bytes> for DataType {
    fn from(bytes: bytes::Bytes) -> Self {
        Self::SimpleType(SimpleType::Blob(Blob(bytes)))
    }
}
impl From<bytes::Bytes> for SimpleType {
    fn from(bytes: bytes::Bytes) -> Self {
        SimpleType::Blob(Blob(bytes))
    }
}
impl From<bytes::Bytes> for Blob {
    fn from(bytes: bytes::Bytes) -> Self {
        Blob(bytes)
    }
}
