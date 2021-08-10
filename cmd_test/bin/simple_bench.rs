use std::time::Instant;

use cmd_test::{read_assert_eq, write_cmd};
use tokio::net::TcpStream;

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let addr = "127.0.0.1:6379";
            let mut streams = vec![];
            let mut v = vec![];
            for _ in 0i32..1000 {
                // Establish a connection to the server
                let stream = TcpStream::connect(addr).await.unwrap();
                streams.push(stream);
            }
            dbg!("connect");
            for i in 0..1000 {
                let stream = streams.pop().unwrap();
                let h = tokio::spawn(key_value_get_set(stream, i));
                v.push(h);
            }
            dbg!("ready");
            let now = Instant::now();
            for ele in v {
                let _ = ele.await;
            }
            dbg!(Instant::now() - now);
        });
}

async fn key_value_get_set(mut stream: TcpStream, suffix: usize) {
    for i in 0..10000 {
        write_cmd(
            &mut stream,
            vec![
                "set",
                &format!("{:049}_{:050}", suffix, i),
                "0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789",
                "EX",
                "300",
            ],
        )
        .await;
        read_assert_eq(&mut stream, b"+OK\r\n").await;
    }
}
