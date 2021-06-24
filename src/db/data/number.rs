use std::ops::Deref;

use super::Data;
use crate::db::{
    result::Result,
    state::{Entry, State},
};

#[derive(Debug, Clone)]
pub struct Number(i64);

impl Deref for Number {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Number {
    fn insert_new(state: &mut State, key: String, value: i64) {
        let id = state.next_id();
        let e = Entry {
            id,
            data: value.into(),
            expires_at: None,
        };
        state.entries.insert(key, e);
    }
}

impl From<i64> for Data {
    fn from(n: i64) -> Self {
        Data::Number(Number(n))
    }
}

impl State {
    pub(crate) fn incr_by(&mut self, key: String, value: i64) -> Result<i64> {
        if let Some(old) = self.entries.get(&key) {
            let (old_value, new_entry) = match &old.data {
                Data::Bytes(b) => {
                    let old_value = std::str::from_utf8(&b[..])
                        .map_err(|e| e.to_string())
                        .and_then(|x| x.parse::<i64>().map_err(|e| e.to_string()))?;
                    (
                        old_value,
                        Entry {
                            id: old.id,
                            expires_at: old.expires_at,
                            data: (value + old_value).into(),
                        },
                    )
                }
                Data::Number(Number(i)) => (
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
