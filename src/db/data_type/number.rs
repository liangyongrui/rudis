use std::ops::Deref;

use super::{DataType, SimpleType};
use crate::db::{
    result::Result,
    slot::{Entry, Slot},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Number(pub i64);

impl Deref for Number {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Number {
    fn insert_new(state: &Slot, key: String, value: i64) {
        let id = state.next_id();
        let e = Entry {
            id,
            data: value.into(),
            expires_at: None,
        };
        state.entries.insert(key, e);
    }
}

impl From<i64> for DataType {
    fn from(n: i64) -> Self {
        DataType::SimpleType(SimpleType::Number(Number(n)))
    }
}

impl From<i64> for SimpleType {
    fn from(n: i64) -> Self {
        SimpleType::Number(Number(n))
    }
}

impl Slot {
    pub(crate) fn incr_by(&self, key: String, value: i64) -> Result<i64> {
        if let Some(old) = self.entries.get(&key) {
            let (old_value, new_entry) = match &old.data {
                DataType::SimpleType(SimpleType::SimpleString(s)) => {
                    let old_value = s.parse::<i64>().map_err(|e| e.to_string())?;
                    (
                        old_value,
                        Entry {
                            id: old.id,
                            expires_at: old.expires_at,
                            data: (value + old_value).into(),
                        },
                    )
                }
                DataType::SimpleType(SimpleType::Number(Number(i))) => (
                    *i,
                    Entry {
                        id: old.id,
                        expires_at: old.expires_at,
                        data: (value + i).into(),
                    },
                ),
                _ => return Err("type not support".to_owned()),
            };
            self.entries.insert(key, new_entry);
            Ok(old_value)
        } else {
            Number::insert_new(self, key, value);
            Ok(0)
        }
    }
}
