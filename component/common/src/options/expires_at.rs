use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ExpiresAt {
    // 指定时间, 0 不会过期
    Specific(u64),
    // 上一次时间
    Last,
}
