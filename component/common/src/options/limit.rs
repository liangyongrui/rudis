use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum Limit {
    // (offset, count)
    //
    // `offset` with -1 being the last element of the sorted set, -2 the penultimate element, and so on.
    //  A negative `count` returns all elements from the `offset`.
    Limit(i64, i64),
    None,
}

impl Default for Limit {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl Limit {
    /// # Errors
    /// no errors
    #[inline]
    pub fn parse_frames(
        tag: &str,
        parse: &crate::connection::parse::Parse,
    ) -> crate::Result<Option<Self>> {
        let res = match tag {
            "limit" => Some(Self::Limit(parse.next_int()?, parse.next_int()?)),
            _ => None,
        };
        Ok(res)
    }
}
