//! # 日志模块

use std::path::Path;

use anyhow::Result;
use log::LevelFilter;
use log4rs::append::{console::ConsoleAppender, file::FileAppender};
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Handle;

/// 日志初始化操作
pub fn init_logger(log_path: &Path) -> Result<Handle> {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h([ {l} ])} {m}{n}")))
        .build();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[ {d(%Y-%m-%dT%H:%M:%S %Z)(utc)} | {M} | {l} ] {m}{n}",
        )))
        .build(log_path.to_string_lossy().to_string().as_str())?;

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(
            Logger::builder()
                .appender("stdout")
                .build("lemmekk", LevelFilter::Debug),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Trace),
        )?;
    let handle = log4rs::init_config(config)?;
    Ok(handle)
}
