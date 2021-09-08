use db::Db;

use crate::Frame;

/// <https://redis.io/commands/info>
#[derive(Debug, Clone)]
pub struct Info;

impl Info {
    #[tracing::instrument(skip(_db))]
    pub fn apply(self, _db: &Db) -> Frame {
        // todo
        Frame::Bulk(
            b"
        # Fake data in `info` command\r\n
        # Server\r\n
        redis_version:0.0.0\r\n
        redis_git_sha1:00000000\r\n
        "[..]
                .into(),
        )
    }
}
