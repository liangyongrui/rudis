use bytes::Bytes;

#[derive(Debug, Clone)]
pub enum Data {
    // bytes 一定是 set 进来的
    Bytes(Bytes),
    Number(i64),
    // todo
    List,
    // todo
    Hash,
    // todo
    Set,
    // todo
    Zset,
}

impl Data {
    pub fn parse_from_bytes(bytes: Bytes) -> Self {
        Self::Bytes(bytes)
    }
}
