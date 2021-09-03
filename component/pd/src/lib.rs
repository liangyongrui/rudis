#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::doc_markdown)]
#![allow(unstable_name_collisions)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::let_underscore_drop)]

use connect::{Handle, Listener};
use connection::Connection;
use status::server_survival_check;
use tokio::net::TcpListener;
use tracing::error;

mod cmd;
/// 连接处理模块
mod connect;
/// 全局状态
mod status;

pub async fn run(listener: TcpListener) -> common::Result<()> {
    tokio::spawn(server_survival_check());
    let mut server = Listener::new(listener);
    loop {
        let socket = server.accept().await?;
        let connection = Connection::new(socket);
        let mut handle = Handle::new(connection);
        tokio::spawn(async move {
            if let Err(err) = handle.run().await {
                error!(cause = ?err, "connection error");
            }
        });
    }
}
