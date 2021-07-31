//! 主要测试 连接过程

use std::{net::SocketAddr, time::Instant};

use cmd_test::{read_assert_eq, write_cmd};
use component::server;
use tokio::net::{TcpListener, TcpStream};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, Registry};

async fn _start_server() -> SocketAddr {
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("rcc test")
        .install_simple()
        .unwrap();

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default().with(telemetry);
    let _ = tracing::subscriber::set_global_default(subscriber);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });

    addr
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // let addr = start_server().await;
            let addr = "127.0.0.1:6379";
            let mut streams = vec![];
            let mut v = vec![];
            for _ in 0i32..1600 {
                let stream = TcpStream::connect(addr).await.unwrap();
                streams.push(stream);
            }
            dbg!("connect");
            for i in 0..1600 {
                let stream = streams.pop().unwrap();
                let h = tokio::spawn(async move {
                    // Establish a connection to the server
                    key_value_get_set(stream, i).await
                });
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
    for i in 0..5000 {
        write_cmd(&mut stream, &format!("SET hello{}_{} world", suffix, i)).await;
        read_assert_eq(&mut stream, b"+OK\r\n").await;
    }
}
