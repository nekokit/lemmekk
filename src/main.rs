use std::{env, process};

use clap::Parser;
use lemmekk::AppError;
use lemmekk::Application;

use lemmekk::CliArgs;

fn main() -> Result<(), AppError> {
    // 创建 App 对象并使用配置文件和执行参数覆盖
    let mut app = match Application::create(CliArgs::parse()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(-1);
        }
    };
    // 设置日志级别
    env::set_var("RUST_LOG", app.config.general.log_level.to_string());
    env_logger::init();
    // 主程序执行
    app.run()
}
