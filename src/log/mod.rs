use std::str::FromStr;
use tracing::*;

use crate::config::NiKoLogConfig;

pub fn init_log(conf: &NiKoLogConfig) {
    let file_appender = tracing_appender::rolling::daily(conf.dir_name(), conf.file_name());
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let level = Level::from_str(conf.level())
        .expect(format!("{} is not a valid log level", conf.level()).as_str());
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_file(true)
        .with_writer(non_blocking)
        .with_level(true)
        .with_max_level(level);
}
