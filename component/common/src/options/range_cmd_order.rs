use std::ops::Bound;

use serde::{Deserialize, Serialize};

use crate::float::Float;

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum RangeCmdOrder {
    Byscore,
    Bylex,
    Byrank,
}

impl Default for RangeCmdOrder {
    fn default() -> Self {
        Self::Byrank
    }
}

impl RangeCmdOrder {
    /// # Errors
    /// str not float
    pub fn parse_float_bound(data: &str) -> crate::Result<Bound<Float>> {
        let res = if data.len() >= 4 && matches!(&data[..3], "-inf" | "+inf") {
            Bound::Unbounded
        } else if let Some(s) = data.strip_prefix('(') {
            Bound::Excluded(Float(s.parse::<f64>()?))
        } else {
            Bound::Included(Float(data.parse::<f64>()?))
        };
        Ok(res)
    }

    /// # Errors
    /// str is empty
    pub fn parse_lex_bound(data: &str) -> crate::Result<Bound<&[u8]>> {
        if data.is_empty() {
            return Err("bound invaild".into());
        }
        let res = if matches!(&data[..1], "-" | "+") {
            Bound::Unbounded
        } else if let Some(s) = data.strip_prefix('(') {
            Bound::Excluded(s.as_bytes())
        } else {
            Bound::Included(data[1..].as_bytes())
        };
        Ok(res)
    }

    /// # Errors
    /// no errors
    #[inline]
    pub fn parse_frames(
        tag: &str,
        _parse: &crate::connection::parse::Parse,
    ) -> crate::Result<Option<Self>> {
        let res = match tag {
            "byscore" => Some(RangeCmdOrder::Byscore),
            "bylex" => Some(RangeCmdOrder::Bylex),
            _ => None,
        };
        Ok(res)
    }
}
