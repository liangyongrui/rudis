use std::{fs::File, io::Read, net::SocketAddr, path::PathBuf};

use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_with::serde_as;
use tracing::{info, warn};

pub static CONFIG: Lazy<Config> = Lazy::new(get_config);

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Maximum number of concurrent connections the redis server will accept.
    ///
    /// When this limit is reached, the server will stop accepting connections until
    /// an active connection terminates.
    #[serde(default = "max_connections")]
    pub max_connections: usize,
    #[serde(default)]
    pub _port: u16,
    #[serde(default)]
    pub _bind: String,
    #[serde(default)]
    pub _timeout: u64,
    #[serde(default)]
    pub _tcp_keepalive: u64,
    #[serde(default)]
    pub _log_level: String,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub master_addr: Option<SocketAddr>,
    #[serde(default = "HdpConfig::default")]
    pub hdp: HdpConfig,
    /// 转发最多积压条数 (aof、主从同步)
    #[serde(default)]
    pub forward_max_backlog: u64,
}

const fn max_connections() -> usize {
    3000
}
/// hdp 相关 配置
#[serde_as]
#[derive(Deserialize, Debug)]
pub struct HdpConfig {
    /// aof 条数达到指定值，触发snapshot，0为不触发
    #[serde(default)]
    pub aof_count: u64,
    /// 保存hdp文件的目录
    pub save_hdp_dir: Option<PathBuf>,
    /// 加载hdp文件的目录
    pub load_hdp_dir: Option<PathBuf>,
}

impl Default for HdpConfig {
    fn default() -> Self {
        Self {
            aof_count: 0,
            save_hdp_dir: None,
            load_hdp_dir: None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct MasterConfig {
    pub ip: String,
    pub port: u16,
}

fn get_config() -> Config {
    info!("loading config");
    // todo 根据环境变量设置配置文件路径
    let file_path = "./config.toml";
    // let file_path = "/Users/liangyongrui/code/github/rcc/conf/config.toml";
    let str_val = match File::open(file_path) {
        Ok(mut file) => {
            let mut str_val = String::new();
            match file.read_to_string(&mut str_val) {
                Ok(s) => s,
                Err(e) => panic!("Error Reading file: {}", e),
            };
            str_val
        }
        Err(e) => {
            warn!("no config.toml file, {}", e);
            String::new()
        }
    };
    let res = toml::from_str(&str_val).unwrap();
    info!("loading config success: {:#?}", res);
    res
}

#[cfg(test)]
mod test {
    use crate::config::CONFIG;
    #[test]
    fn test() {
        let _ = tracing_subscriber::fmt::Subscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
        let _ = &*CONFIG;
    }
}
