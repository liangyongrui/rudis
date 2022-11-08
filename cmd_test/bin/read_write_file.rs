//! 主要测试 连接过程

use std::time::Duration;

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    time::sleep,
};

#[tokio::main]
async fn main() {
    tokio::spawn(write());
    sleep(Duration::from_secs(1)).await;
    read().await;
}

async fn read() {
    let mut f = File::open("./target/test.test").await.unwrap();
    dbg!(f.metadata().await.unwrap());
    dbg!(f.metadata().await.unwrap().len());
    let mut buffer = [0; 10];
    loop {
        sleep(Duration::from_secs(1)).await;
        f.read_exact(&mut buffer).await.unwrap();
        let s: String = buffer
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        dbg!(s);
    }
}

async fn write() {
    let mut f = File::create("./target/test.test").await.unwrap();
    let init = [2; 10];
    f.write_all(&init).await.unwrap();
    sleep(Duration::from_secs(1)).await;
    loop {
        sleep(Duration::from_secs(1)).await;
        let buffer = [1; 10];
        f.write_all(&buffer).await.unwrap();
    }
}
