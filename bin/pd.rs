use structopt::StructOpt;
use tokio::net::TcpListener;

pub const DEFAULT_PORT: &str = "6380";

pub fn main() -> common::Result<()> {
    // enable logging
    // see https://docs.rs/tracing for more info
    let _ = tracing_subscriber::fmt::Subscriber::builder()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_max_level(tracing::Level::INFO)
        .try_init();

    let cli = Cli::from_args();
    let port = cli.port.as_deref().unwrap_or(DEFAULT_PORT);
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;
            pd::run(listener).await
        })
}

#[derive(StructOpt, Debug)]
#[structopt(name = "rcc-pd", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = "A Redis server placement driver")]
struct Cli {
    #[structopt(name = "port", long = "--port")]
    port: Option<String>,
}
