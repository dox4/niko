use std::str::FromStr;
use tracing::*;
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::global_config;

pub fn init_log() {
    let conf = global_config().log();
    // log to console
    let console_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_file(false)
        .with_line_number(true)
        .with_writer(std::io::stdout);

    // log to rolling files
    let file_appender = tracing_appender::rolling::daily(conf.dir_name(), conf.file_name());
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // TODO log could not be written to the target file
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_line_number(true)
        .with_file(true)
        .with_writer(non_blocking)
        .with_level(true);
    let level = Level::from_str(conf.level())
        .expect(format!("{} is not a valid log level", conf.level()).as_str());
    let filter = LevelFilter::from_level(level);
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();
}
