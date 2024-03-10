//！ # 让我康康 Cli 模块
//！
//！ 这是一个基于 7zip 的批量解压的工具，可使用密钥文件匹配压缩包密码。
//！ 支持的压缩格式有 rar4, rar5, zip, 7z, xz 等，同时支持图种检测、递归解压并提供密钥管理等功能。

use std::process;

use clap::Parser;
use log::error;

use lemmekk::{init_logger, AppInfo, Cli, CliArgs, DEFAULT_PATH};

fn main() {
    // 初始化日志
    let _ = match init_logger(&DEFAULT_PATH.log) {
        Ok(handle) => handle,
        Err(_) => {
            eprintln!("日志初始化失败");
            process::exit(-1);
        }
    };

    // 输出版本信息
    let mut appinfo = AppInfo::default();
    appinfo.set_module_name("Cli");
    appinfo.set_module_version(0, 1, 0, "Dev");
    // println!("{}", appinfo.display());

    // 创建 App 对象并使用执行参数覆盖配置文件
    let mut cli = match Cli::create(CliArgs::parse()) {
        Ok(v) => v,
        Err(e) => {
            error!("{:?}", e);
            process::exit(-1);
        }
    };

    // 主程序执行
    if let Err(e) = cli.startup() {
        error!("{:?}", e);
        process::exit(-1);
    }
}
