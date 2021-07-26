use std::{fs::File, io::Read, net::SocketAddr, path::PathBuf, time::Duration};

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
    #[serde(default)]
    pub max_connections: usize,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub bind: String,
    #[serde(default)]
    pub timeout: u64,
    #[serde(default)]
    pub tcp_keepalive: u64,
    #[serde(default)]
    pub log_level: String,
    #[serde(default)]
    pub replica: bool,
    #[serde(default)]
    pub master_addr: Option<SocketAddr>,
    #[serde(default = "HdpConfig::default")]
    pub hdp: HdpConfig,
    /// 转发最多积压条数 (aof、主从同步)
    pub forward_max_backlog: u64,
}

/// hdp 相关 配置
#[serde_as]
#[derive(Deserialize, Debug)]
pub struct HdpConfig {
    /// 每隔x秒，至少有y条数据发现变化，触发 bg_save
    #[serde(default)]
    #[serde_as(as = "Vec<(serde_with::DurationSeconds, _)>")]
    pub save_frequency: Vec<(Duration, u64)>,
    /// 保存hdp文件的目录
    pub save_hdp_dir: Option<PathBuf>,
    /// 加载hdp文件的目录
    pub load_hdp_dir: Option<PathBuf>,
}

impl Default for HdpConfig {
    fn default() -> Self {
        Self {
            save_frequency: vec![(Duration::from_secs(600), 1)],
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
    // let file_path = "./config.toml";
    let file_path = "/Users/liangyongrui/code/github/rcc/conf/config.toml";
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
