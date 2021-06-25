mod blob;
use std::convert::TryFrom;

pub use self::blob::Blob;

mod number;
use self::number::Number;

mod list;
use self::list::List;

#[derive(Debug, Clone)]
pub enum DataType {
    SimpleType(SimpleType),
    AggregateType(AggregateType), // Bytes(Blob),
                                  // Number(Number),
                                  // List(List),
                                  // // todo
                                  // Hash,
                                  // // todo
                                  // Set,
                                  // // todo
                                  // SortedSet
}
#[derive(Debug, Clone)]
pub enum SimpleType {
    Blob(Blob),
    SimpleString(String),
    Number(Number),
    // Bool(bool),
    // todo
    VerbatimString,
    // todo
    BigNumber,
}

#[derive(Debug, Clone)]
pub enum AggregateType {
    List(List),
    Map,
    Set,
    SortedSet,
}

impl TryFrom<DataType> for SimpleType {
    type Error = &'static str;

    fn try_from(value: DataType) -> Result<Self, Self::Error> {
        match value {
            DataType::SimpleType(s) => Ok(s),
            _ => Err("类型不对"),
        }
    }
}
impl From<SimpleType> for DataType {
    fn from(s: SimpleType) -> Self {
        DataType::SimpleType(s)
    }
}
impl From<AggregateType> for DataType {
    fn from(s: AggregateType) -> Self {
        DataType::AggregateType(s)
    }
}
