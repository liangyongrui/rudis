mod bytes;
pub use self::bytes::Bytes;

mod number;
use self::number::Number;

mod list;
use self::list::List;

#[derive(Debug, Clone)]
pub enum Data {
    // bytes 一定是 set 进来的
    Bytes(bytes::Bytes),
    Number(Number),
    List(List),
    // todo
    Hash,
    // todo
    Set,
    // todo
    SortedSet,
}
