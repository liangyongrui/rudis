use std::{fs::File, io::Read, net::SocketAddr, path::PathBuf, time::Duration};

use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_with::serde_as;
use tracing::{info, warn};

pub static CONFIG: Lazy<Config> = Lazy::new(get_config);

fn get_config() -> Config {
    info!("loading config");
    // todo path
    // let file_path = "./config.toml";
    let file_path = "/Users/liangyongrui/code/github/rcc/tests/config.toml";
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
#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
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

    /// 每隔x秒，至少有y条数据发现变化，触发 bg_save
    #[serde(default)]
    #[serde_as(as = "Vec<(serde_with::DurationSeconds, _)>")]
    pub save_hds: Vec<(Duration, u64)>,
    /// 持久化文件保存路径
    #[serde(default)]
    pub save_hds_path: PathBuf,
    /// 是否要从hds文件中加载
    pub load_hds_path: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
pub struct MasterConfig {
    pub ip: String,
    pub port: u16,
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
