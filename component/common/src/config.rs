use std::{env, net::SocketAddr, process::exit};

use once_cell::sync::Lazy;
use serde::Deserialize;
use tracing::{debug, error, info};

pub static CONFIG: Lazy<Config> = Lazy::new(|| match get_config() {
    Ok(c) => {
        info!("loading config success: {:#?}", c);
        c
    }
    Err(e) => {
        error!("init config: {:#?}", e);
        exit(-1)
    }
});

#[derive(Deserialize, Debug)]
pub struct Config {
    /// 当前服务启动地址
    pub server_addr: SocketAddr,
    /// 转发服务的地址
    pub forward_addr: SocketAddr,

    /// Maximum number of concurrent connections the redis server will accept.
    ///
    /// When this limit is reached, the server will stop accepting connections until
    /// an active connection terminates.
    pub max_connections: usize,

    /// 是否从pd初始化
    ///
    /// 默认不走pd
    pub from_pd: Option<Pd>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Pd {
    pub addr: SocketAddr,
    pub group_id: usize,
}

#[derive(Deserialize, Debug)]
pub struct Master {
    pub ip: String,
    pub port: u16,
}

fn get_config() -> crate::Result<Config> {
    info!("loading config");

    let name = env::var("RCC_CONFIG").unwrap_or_else(|_| "config".into());
    let config_file = config::File::with_name(&name).required(false);
    debug!("config_file: {:#?}", config_file);

    let mut settings = config::Config::default();
    settings
        .merge(config_file)?
        .merge(config::Environment::with_prefix("RCC"))?
        .set_default("max_connections", 3000)?
        .set_default("server_addr", "0.0.0.0:6379")?
        .set_default("forward_addr", "0.0.0.0:0")?;

    settings.try_into().map_err(|t| t.into())
}

#[cfg(test)]
mod test {
    use crate::config::CONFIG;
    #[test]
    fn test() {
        #[allow(clippy::let_underscore_drop)]
        let _ = tracing_subscriber::fmt::Subscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
        let _ = &*CONFIG;
    }
}
