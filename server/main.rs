//! rcc server.
//!
//! This file is the entry point for the server implemented in the library. It
//! performs command line parsing and passes the arguments on to
//! `rcc::server`.
//!
//! The `clap` crate is used for parsing arguments.

use component::{server, DEFAULT_PORT};
use structopt::StructOpt;
use tokio::{net::TcpListener, signal};

pub fn main() -> component::Result<()> {
    // enable logging
    // see https://docs.rs/tracing for more info
    // tracing_subscriber::fmt::try_init()?;
    // tracing_subscriber::fmt::Subscriber::builder()
    //     .with_max_level(Level::DEBUG)
    //     .try_init()?;

    let cli = Cli::from_args();
    let port = cli.port.as_deref().unwrap_or(DEFAULT_PORT);
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Bind a TCP listener
            let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;

            server::run(listener, signal::ctrl_c()).await
        })
}

#[derive(StructOpt, Debug)]
#[structopt(name = "rcc-server", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = "A Redis server")]
struct Cli {
    #[structopt(name = "port", long = "--port")]
    port: Option<String>,
}
