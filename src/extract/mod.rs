//! # 解压模块

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

mod job;
mod manager;

pub use job::{ExtractJob, ExtractJobKind};
pub use manager::Extractor;

/// # 解压配置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtractMethod {
    /// 识别图片隐写文件
    pub recogniz_steganography: bool,
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
            recogniz_steganography: false,
            extract_directly: false,
            smart_directly: false,
            recursively: false,
        }
    }
}

/// # 解压后操作
#[derive(Clone, Debug, Serialize, Deserialize, ValueEnum)]
pub enum DeferOperation {
    /// 什么都不做
    DoNothing,
    /// 删除
    Delete,
    /// 移动
    Move,
}
