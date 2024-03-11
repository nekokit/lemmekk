//! # 特殊内容提供模块

use std::cell::LazyCell;
use std::env;
use std::path::PathBuf;

use anyhow::Context;
use regex::Regex;

/// 默认路径提供器
#[derive(Debug)]
pub struct DefaultPath {
    pub data_dir: PathBuf,
    pub config: PathBuf,
    pub log: PathBuf,
    pub token: PathBuf,
    pub token_convert: PathBuf,
}

impl Default for DefaultPath {
    fn default() -> Self {
        let data_dir = match env::consts::OS {
            "windows" => get_self_dir(),
            _ => get_data_dir(),
        };
        let config = match env::consts::OS {
            "windows" => get_self_dir().join("config.toml"),
            _ => get_data_dir().join("config.toml"),
        };
        let log = match env::consts::OS {
            "windows" => get_self_dir().join("result.log"),
            _ => get_data_dir().join("result.log"),
        };
        let token = match env::consts::OS {
            "windows" => get_self_dir().join("default.token"),
            _ => get_data_dir().join("default.token"),
        };
        let convert_token = match env::consts::OS {
            "windows" => get_self_dir().join("token.txt"),
            _ => get_data_dir().join("token.txt"),
        };
        Self {
            data_dir,
            config,
            log,
            token,
            token_convert: convert_token,
        }
    }
}

/// 获取执行文件所在目录
fn get_self_dir() -> PathBuf {
    let mut dir = env::current_exe().expect("读取程序路径错误");
    dir.pop();
    dir
}

/// 获取系统配置目录
///
/// 用于 Linux 系统，返回 `~/.config` 的绝对路径
fn get_data_dir() -> PathBuf {
    dirs::config_dir()
        .context("无法读取系统配置目录")
        .unwrap()
        .join("lemmekk")
}

/// 默认路径提供器
pub const DEFAULT_PATH: LazyCell<DefaultPath> = LazyCell::new(|| DefaultPath::default());

/// 默认正则提供器
#[derive(Debug)]
pub struct DefaultRegex {
    pub token_file_pattern_jtmdy: Regex,
}
impl Default for DefaultRegex {
    fn default() -> Self {
        Self {
            token_file_pattern_jtmdy: Regex::new(r"\b(?<token>.+)\t\t(?<count>\d+)\b").unwrap(),
        }
    }
}

/// 默认正则提供器
pub const DEFAULT_REGEX: LazyCell<DefaultRegex> = LazyCell::new(|| DefaultRegex::default());
