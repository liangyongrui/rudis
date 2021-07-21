use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::data_type::{DataType, SimpleType};

pub struct Dict {
    pub inner: HashMap<SimpleType, Value>,
}

pub struct Value {
    pub id: u64,
    pub data: DataType,
    pub expire_at: Option<DateTime<Utc>>,
}
