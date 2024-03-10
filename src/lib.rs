//！ # 让我康康
//!
//！ 这是一个基于 7zip 的批量解压的工具，可使用密钥文件匹配压缩包密码。
//！ 支持的压缩格式有 rar4, rar5, zip, 7z, xz 等，同时支持图种检测、递归解压并提供密钥管理等功能。

#![feature(lazy_cell)]

mod appinfo;
mod cli;
mod config;
mod log;
mod token;

pub use appinfo::AppInfo;
pub use cli::{Cli, CliArgs, MainCommand, TokenProcess};
pub use config::{Config, TokenFilePattern, TokenListStyle, DEFAULT_PATH};
pub use log::init_logger;
pub use token::TokenManager;
