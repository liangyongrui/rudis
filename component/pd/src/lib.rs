use common::connection::Connection;
use connect::{Handle, Listener};
use status::server_survival_check;
use tokio::net::TcpListener;
use tracing::error;

mod cmd;
/// 连接处理模块
mod connect;
/// 全局状态
mod status;

/// run pd
/// # Errors
/// io errors
#[inline]
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
