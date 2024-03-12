//! # 解压模块

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

mod manager;

pub use manager::Extractor;

/// # 解压配置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtractMethod {
    /// 识别图片隐写文件
    pub analyze_steganography: bool,
    /// 直接解压
    pub extract_directly: bool,
    /// 智能直接解压
    pub smart_directly: bool,
    /// 递归解压
    pub recursively: bool,
}
impl Default for ExtractMethod {
    fn default() -> Self {
        Self {
            analyze_steganography: false,
            extract_directly: false,
            smart_directly: false,
            recursively: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ValueEnum)]
pub enum DeferOperation {
    DoNothing,
    Delete,
    Move,
}
