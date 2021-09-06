#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use common::config::CONFIG;
use tokio::{net::TcpListener, signal};

pub fn main() -> common::Result<()> {
    // enable logging
    // see https://docs.rs/tracing for more info
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_max_level(tracing::Level::INFO)
        .try_init();

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let listener = TcpListener::bind(CONFIG.server_addr).await?;
            server::run(listener, signal::ctrl_c()).await
        })
}
