use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NiKoConfig<'n> {
    #[serde(borrow)]
    pub db: NiKoDbConfig<'n>,
}

#[derive(Debug, Deserialize)]
pub struct NiKoDbConfig<'a> {
    pub host: &'a str,
    pub port: u32,
    pub user: &'a str,
    pub password: &'a str,
    pub db_type: &'a str,
    pub db_name: &'a str,
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

// pub fn from_file(path: &str) -> NiKoConfig {
//     let filebytes = fs::read(path).expect("read config file failed");
//     let c = toml::from_slice(&filebytes[..]);
//     match c {
//         Ok(nk) => nk,
//         Err(err) => panic!("read config failed with err: {:?}", err),
//     }
// }

pub fn from_slice(src: &[u8]) -> NiKoConfig {
    toml::from_slice(src).expect("read config failed.")
}
