//! # 特殊内容提供模块

use std::cell::LazyCell;
use std::env;
use std::path::PathBuf;

use anyhow::Context;
use regex::{Regex, RegexBuilder};

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
    /// 解TMD压 密码本模式
    pub token_file_pattern_jtmdy: Regex,
    /// 7zip 版本
    pub version_7z: Regex,
    /// 分卷压缩包名
    pub split_pack_name: Vec<Regex>,
}
impl Default for DefaultRegex {
    fn default() -> Self {
        Self {
            token_file_pattern_jtmdy: Regex::new(r"\b(?<token>.+)\t\t(?<count>\d+)\b").unwrap(),
            version_7z: RegexBuilder::new(r"\b7-Zip.*? (?<version>\d+.\d+) ")
                .case_insensitive(true)
                .build()
                .unwrap(),
            split_pack_name: vec![
                RegexBuilder::new(r"^(?<package>.*)\.part(?<vol>\d+)\.(:?rar|exe)$")
                    .case_insensitive(true)
                    .build()
                    .unwrap(),
                RegexBuilder::new(r"^(?<package>.*)\.(?:7z|zip|tar)\.(?<vol>\d{3,})$")
                    .case_insensitive(true)
                    .build()
                    .unwrap(),
                RegexBuilder::new(r"^(?<package>.*)\.z(?<vol>\d{2,})$")
                    .case_insensitive(true)
                    .build()
                    .unwrap(),
            ],
        }
    }
}

/// 默认正则提供器
pub const DEFAULT_REGEX: LazyCell<DefaultRegex> = LazyCell::new(|| DefaultRegex::default());

/// 魔数提供器
pub const COVER_FEATURE: [(&str, &[u8], &[u8]); 4] = [
    ("jpg", &[0xFF, 0xD8, 0xFF], &[0xFF, 0xD9]),
    ("png", &[0x89, 0x50, 0x4E, 0x47], &[0xAE, 0x42, 0x60, 0x82]),
    ("gif", &[0x47, 0x49, 0x46, 0x38, 0x39, 0x61], &[0x00, 0x3B]),
    ("gif", &[0x47, 0x49, 0x46, 0x38, 0x37, 0x61], &[0x00, 0x3B]),
];
pub const STEGO_FEATURE: [(&str, &[u8]); 12] = [
    ("zip", &[0x50, 0x4B, 0x03, 0x04]),
    ("rar5", &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00]),
    ("rar4", &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00]),
    ("7z", &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
    ("tar", &[0x75, 0x73, 0x74, 0x61, 0x72]),
    ("xz", &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]),
    ("targz", &[0x1F, 0x8B, 0x08, 0x00]),
    ("gz", &[0x1F, 0x8B]),
    ("tarbz", &[0x42, 0x5A, 0x68, 0x39, 0x17]),
    ("bz2", &[0x42, 0x5A, 0x68]),
    ("bz", &[0x42, 0x5A]),
    ("z", &[0x1F, 0x9D]),
];
