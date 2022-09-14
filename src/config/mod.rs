use std::net::{IpAddr, SocketAddr};

use getset::Getters;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Getters)]
pub struct NiKoConfig<'n> {
    #[serde(borrow)]
    #[get = "pub"]
    db: NiKoDbConfig<'n>,
    #[serde(borrow)]
    #[get = "pub"]
    server: NiKoServerConfig<'n>,
    #[serde(borrow)]
    #[get = "pub"]
    log: NiKoLogConfig<'n>,
    #[serde(borrow)]
    #[get = "pub"]
    dir: NiKoDirConfig<'n>,
}

#[derive(Debug, Deserialize, Getters)]
pub struct NiKoDbConfig<'a> {
    host: &'a str,
    port: u32,
    user: &'a str,
    password: &'a str,
    db_type: &'a str,
    db_name: &'a str,
}

#[derive(Debug, Deserialize, Getters)]
pub struct NiKoServerConfig<'a> {
    host: &'a str,
    port: u16,
}

#[derive(Debug, Deserialize, Getters)]
pub struct NiKoLogConfig<'a> {
    #[get = "pub"]
    level: &'a str,
    #[get = "pub"]
    dir_name: &'a str,
    #[get = "pub"]
    file_name: &'a str,
}

#[derive(Debug, Deserialize, Getters)]
pub struct NiKoDirConfig<'a> {
    #[get = "pub"]
    full_path: &'a str,
}

impl NiKoDbConfig<'_> {
    pub fn db_url(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.db_type, self.user, self.password, self.host, self.port, self.db_name
        )
    }

    pub fn db_type(&self) -> String {
        self.db_type.to_string()
    }
}

impl NiKoServerConfig<'_> {
    pub fn sock(&self) -> SocketAddr {
        SocketAddr::new(
            self.host.parse().expect("expect a valid IP address."),
            self.port,
        )
    }
}

pub fn from_slice(src: &[u8]) -> NiKoConfig {
    toml::from_slice(src).expect("read config failed.")
}
