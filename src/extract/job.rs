//! # 解压任务模块

use std::path::PathBuf;

/// # 解压任务
#[derive(Debug, Default)]
pub struct ExtractJob {
    pub package: String,
    pub kind: ExtractJobKind,
    pub path: PathBuf,
    pub token: String,
    pub relevant: Vec<PathBuf>,
}

/// # 解压任务类型
#[derive(Debug)]
pub enum ExtractJobKind {
    Split(usize),
    Stego,
    Normal,
}
impl Default for ExtractJobKind {
    fn default() -> Self {
        Self::Normal
    }
}
