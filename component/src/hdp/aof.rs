use std::path::Path;

use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

use crate::forward::Message;

/// 每个 slot 都有一个aof_status
pub struct AofStatus {
    pub cur_id: u64,
    pub file: BufWriter<File>,
}

impl AofStatus {
    pub async fn new(save_hdp_dir: &Path) -> crate::Result<Self> {
        let save_path = save_hdp_dir; // todo
        let display = save_path.display();
        match File::create(save_path).await {
            Err(why) => Err(format!("couldn't create {}: {}", display, why).into()),
            Ok(file) => Ok(Self {
                file: BufWriter::new(file),
                // todo
                cur_id: 0,
            }),
        }
    }
    pub async fn flush(&mut self) -> crate::Result<()> {
        self.file.flush().await.map_err(|t| t.into())
    }
    pub async fn write(&mut self, message: &Message) -> crate::Result<()> {
        let next_id = self.cur_id + 1;
        match next_id.cmp(&message.id) {
            // 从buf中追回
            std::cmp::Ordering::Less => todo!(),
            // 写入文件
            std::cmp::Ordering::Equal => message.stream_encode(&mut self.file).await?,
            // 忽略不处理
            std::cmp::Ordering::Greater => (),
        }
        Ok(())
    }
}
