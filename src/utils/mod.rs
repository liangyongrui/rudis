pub mod custom_serde;
pub mod options;
pub mod pointer;

use std::ops::Bound;

use serde::{Deserialize, Serialize};
pub trait BoundExt<T> {
    fn map<O, F: FnOnce(T) -> O>(self, f: F) -> Bound<O>;
}

impl<T> BoundExt<T> for Bound<T> {
    fn map<O, F: FnOnce(T) -> O>(self, f: F) -> Bound<O> {
        match self {
            Bound::Included(t) => Bound::Included(f(t)),
            Bound::Excluded(t) => Bound::Excluded(f(t)),
            Bound::Unbounded => Bound::Unbounded,
        }
    }
}

pub fn u8_to_string(data: &[u8]) -> String {
    std::str::from_utf8(data)
        .map(|s| s.to_string())
        .expect("protocol error; invalid string")
}

pub fn u8_to_i64(data: &[u8]) -> i64 {
    atoi::atoi::<i64>(data).expect("protocol error; invalid number")
}

pub trait ParseSerdeType<'de, T: Deserialize<'de> + Serialize> {
    fn parse_serde_type(&self) -> T;
}
