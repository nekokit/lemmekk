//！ # 让我康康
//！ 这是一个基于 7zip 的批量解压的工具，可使用密码字典匹配压缩包密码。
//！ 支持的压缩格式有 rar4, rar5, zip, 7z, xz 等，同时支持图种检测、递归解压、密码管理等功能。
#![feature(lazy_cell)]

use regex::Regex;
use std::{cell::LazyCell, env, path::PathBuf};

mod app;
mod config;
// mod error;
mod log;
mod password;

pub use app::Application;
pub use app::CliArgs;
// pub use error::AppError;
pub use filepattern::CompressType;
pub use filepattern::ImageType;
pub use filepattern::COMPRESSED_FEATURE;
pub use filepattern::IMAGE_FEATURE;
pub use password::Password;
pub use password::PasswordFile;
pub use password::PasswordList;

pub mod filepattern;
pub mod utils;

/// 默认路径提供器
pub const DEFAULT_PATH: LazyCell<PathLoader> = LazyCell::new(|| PathLoader::default());

/// 默认正则提供器
pub const DEFAULT_REGEX: LazyCell<RegexLoader> = LazyCell::new(|| RegexLoader {
    pw_type_jtmdy: Regex::new(r"(.+)\t\t(\d+)").unwrap(),
    version_7z: Regex::new(r"7-Zip.+ (\d+.\d+) :").unwrap(),
});

/// # 路径提供器
/// 调用方法返回相应的路径。
pub struct PathLoader {
    pub program: PathBuf,
    pub pwd: PathBuf,
    pub config: PathBuf,
    pub password: PathBuf,
    pub log: PathBuf,
    pub convert: PathBuf,
}
impl Default for PathLoader {
    fn default() -> Self {
        let mut program = env::current_exe().expect("读取程序路径错误");
        program.pop();
        let pwd = env::current_dir().expect("读取工作路径错误");
        let config = program.clone().join("config.toml");
        let password = program.clone().join("passwords.toml");
        let log = program.clone().join("result.log");
        let convert = program.clone().join("passwords.txt");
        Self {
            program,
            pwd,
            config,
            password,
            log,
            convert,
        }
    }
}

pub struct RegexLoader {
    pw_type_jtmdy: Regex,
    version_7z: Regex,
}
