//! # 解压管理模块
//!
//! 提供解压管理器，用于操作识别解压文件、生成解压任务、解压任务的解析与调度。

use log::{debug, info, warn};
use std::{collections::HashMap, ffi::OsStr, path::PathBuf, process};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::{config::ExtractConfig, ExtractJob, ExtractJobKind, DEFAULT_REGEX};

/// # 解压管理器
#[derive(Debug, Default)]
pub struct Extractor {
    config: ExtractConfig,
    files: Vec<PathBuf>,
}

impl Extractor {
    /// 解压管理器初始化
    pub fn load(&mut self, config: ExtractConfig) -> Result<()> {
        self.config = config;
        // todo 测试7z
        self.test_7z().context("测试 7z 命令失败")?;
        self.load_files();
        Ok(())
    }

    /// 测试 7z 命令
    fn test_7z(&self) -> Result<()> {
        let command_7z = self.config.get_command_7z();
        debug!("7z 调用：{}", command_7z.display());
        let output = process::Command::new(&command_7z).arg("--help").output()?;
        match DEFAULT_REGEX
            .version_7z
            .captures(&String::from_utf8(output.stdout)?)
        {
            Some(cap) => println!("7z ver.{}", &cap["version"]),
            None => warn!("未知的 7z 版本"),
        };
        Ok(())
    }

    /// 添加待分析文件
    ///
    /// 从输入将载符合条件的文件添加到管理器
    fn load_files(&mut self) {
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
        self.files = files
    }

    /// 生成解压任务
    ///
    /// 根据载入的文件列表，合并分卷并识别隐写文件
    pub fn create_extract_job(&mut self) {
        // 载入任务，识别分卷
        let jobs_unchecked = self
            .files
            .iter()
            .fold(
                HashMap::new(),
                |mut acc: HashMap<(bool, PathBuf, String), ExtractJob>, item| {
                    // 为不同文件夹的文件创建识别码
                    let filename = match item.file_name().and_then(OsStr::to_str) {
                        Some(v) => v.to_string(),
                        None => {
                            warn!("转换为任务失败：{}", item.display());
                            return acc;
                        }
                    };
                    let mut dir_path = item.to_path_buf();
                    dir_path.pop();

                    // 匹配分卷文件名
                    let matched_pattern =
                        DEFAULT_REGEX.split_pack_name.iter().fold(None, |p, r| {
                            if r.is_match(&filename) {
                                Some(r.clone())
                            } else {
                                p
                            }
                        });
                    if let Some(r) = matched_pattern {
                        // 如果与分卷压缩名匹配
                        if let Some(cap) = r.captures(&filename) {
                            let package = cap["package"].to_string();
                            let volume = match cap["vol"].to_string().parse::<usize>() {
                                Ok(v) => v,
                                Err(e) => {
                                    warn!("解析分卷出错[{}]: {}", e, item.display());
                                    return acc;
                                }
                            };
                            // key(是否分卷,目录,包名)
                            let key = (true, dir_path.to_path_buf(), package.clone());

                            match acc.get_mut(&key) {
                                // 已存在同名分卷
                                Some(value) => {
                                    info!(
                                        "文件 [{}] 已加入分卷压缩列表 {}",
                                        item.display(),
                                        value.package
                                    );
                                    if volume == 1 {
                                        // 如果是第 1 卷则加入解压路径
                                        value.path = item.to_path_buf();
                                    } else {
                                        // 如果不是第 1 卷则加入更新最大卷并加入关联列表
                                        match &mut value.kind {
                                            crate::ExtractJobKind::Split(v) => {
                                                if &volume > v {
                                                    *v = volume
                                                }
                                            }
                                            _ => {}
                                        };
                                        value.relevant.push(item.to_path_buf());
                                    }
                                }

                                // 不存在同名分卷则转为任务添加进 hashmap
                                None => {
                                    info!(
                                        "通过文件 [{}] 创建解压任务：{} ",
                                        item.display(),
                                        package
                                    );
                                    acc.insert(
                                        key.clone(),
                                        ExtractJob {
                                            package,
                                            kind: ExtractJobKind::Split(volume),
                                            path: match volume {
                                                1 => item.to_path_buf(),
                                                _ => PathBuf::new(),
                                            },
                                            token: String::new(),
                                            relevant: match volume {
                                                1 => vec![],
                                                _ => vec![item.to_path_buf()],
                                            },
                                        },
                                    );
                                }
                            };
                        }
                    } else {
                        // 如果与分卷文件名不匹配
                        // key(是否分卷,目录,包名)
                        let key = (false, dir_path.to_path_buf(), filename.clone());
                        let package =
                            match item.with_extension("").file_name().and_then(OsStr::to_str) {
                                Some(s) => s.to_string(),
                                None => {
                                    warn!("转换为任务失败：{}", item.display());
                                    return acc;
                                }
                            };
                        acc.insert(
                            key,
                            ExtractJob {
                                package,
                                kind: ExtractJobKind::Normal,
                                path: item.to_path_buf(),
                                token: String::new(),
                                relevant: vec![],
                            },
                        );
                    }

                    acc
                },
            )
            .into_values()
            .collect::<Vec<ExtractJob>>();

        info!(
            "载入解压任务完成：[\n{}\n]",
            jobs_unchecked
                .iter()
                .map(|j| j.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        );

        // todo 处理隐写
        // if self.config.method.analyze_steganography {}
    }
}
