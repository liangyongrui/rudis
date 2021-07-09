use super::{DataType, SimpleType};

impl From<bytes::Bytes> for DataType {
    fn from(bytes: bytes::Bytes) -> Self {
        Self::SimpleType(SimpleType::Blob(bytes))
    }
}
impl From<bytes::Bytes> for SimpleType {
    fn from(bytes: bytes::Bytes) -> Self {
        SimpleType::Blob(bytes)
    }
}
