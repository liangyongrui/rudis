use super::{DataType, SimpleType};
use crate::db::{dict, result::Result, slot::Slot};

impl From<i64> for DataType {
    fn from(n: i64) -> Self {
        DataType::SimpleType(SimpleType::Integer(n))
    }
}

impl From<i64> for SimpleType {
    fn from(n: i64) -> Self {
        SimpleType::Integer(n)
    }
}

impl Slot {
    pub fn incr_by(&self, key: SimpleType, value: i64) -> Result<i64> {
        self.dict
            .get_or_insert(
                key,
                || dict::Entry {
                    id: self.next_id(),
                    expires_at: None,
                    data: 0.into(),
                },
                |entry| {
                    let (after_value, new_entry) = match &entry.data {
                        DataType::SimpleType(SimpleType::SimpleString(s)) => {
                            let after_value = s.parse::<i64>().map_err(|e| e.to_string())? + value;
                            (
                                after_value,
                                dict::Entry {
                                    id: entry.id,
                                    expires_at: entry.expires_at,
                                    data: after_value.into(),
                                },
                            )
                        }
                        DataType::SimpleType(SimpleType::Integer(i)) => (
                            value + i,
                            dict::Entry {
                                id: entry.id,
                                expires_at: entry.expires_at,
                                data: (value + i).into(),
                            },
                        ),
                        _ => return Err("type not support".to_owned()),
                    };
                    *entry = new_entry;
                    Ok(after_value)
                },
            )
            .0
    }
}

#[cfg(test)]
mod test {
    use crate::{db::slot::Slot, utils::options::NxXx};

    #[tokio::test]
    async fn test() {
        let _ = tracing_subscriber::fmt::Subscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
        let slot = Slot::new();
        assert_eq!(slot.incr_by("abc".into(), 123), Ok(123));
        assert_eq!(slot.incr_by("abc".into(), 123), Ok(123 + 123));
        slot.set("aaa".into(), "2345".into(), NxXx::None, None, false)
            .await
            .unwrap();
        assert_eq!(slot.incr_by("aaa".into(), -123), Ok(2345 - 123));
    }
}
