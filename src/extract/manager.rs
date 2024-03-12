//! # 解压管理模块
//!
//! 提供解压管理器，用于操作识别解压文件、生成解压任务、解压任务的解析与调度。

use std::{ffi::OsStr, path::PathBuf};

use anyhow::Result;
use log::{debug, info, warn};
use walkdir::WalkDir;

use crate::config::ExtractConfig;

/// # 解压管理器
#[derive(Debug, Default)]
pub struct Extractor {
    config: ExtractConfig,
    unchecked_files: Vec<PathBuf>,
}

impl Extractor {
    /// 解压管理器初始化
    pub fn load(&mut self, config: ExtractConfig) -> Result<()> {
        debug!("初始化解压管理器");
        self.config = config;
        self.load_unchecked_files();
        Ok(())
    }

    /// 添加待分析文件
    ///
    /// 从输入将载符合条件的文件添加到管理器
    fn load_unchecked_files(&mut self) {
        let mut files = self
            .config
            .source
            .iter()
            .fold(vec![], |mut acc: Vec<PathBuf>, item| {
                if item.is_file() {
                    debug!("输入源为文件：{}", item.display());
                    acc.push(item.to_path_buf());
                }
                if item.is_dir() {
                    debug!("输入源为文件夹：{}", item.display());
                    WalkDir::new(item).into_iter().for_each(|r| match r {
                        Ok(entry) => {
                            if entry.path().is_file() {
                                acc.push(entry.path().to_path_buf());
                            }
                        }
                        Err(e) => {
                            warn!("无法读取输入源文件夹 `{}`: {}", item.display(), e)
                        }
                    })
                }
                return acc;
            });
        info!("{:#?}", files);
        if self.config.excluded_suffix.len() > 0 {
            debug!("过滤扩展名：{:?}", self.config.excluded_suffix);
            files = files
                .into_iter()
                .filter(|item| {
                    !self.config.excluded_suffix.contains(
                        &item
                            .extension()
                            .and_then(OsStr::to_str)
                            .unwrap_or("")
                            .to_string(),
                    )
                })
                .collect()
        };
        self.unchecked_files = files
    }
}
