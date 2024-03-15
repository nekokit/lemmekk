//! # 解压管理模块
//!
//! 提供解压管理器，用于操作识别解压文件、生成解压任务、解压任务的解析与调度。

use log::{debug, info, warn};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process,
};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::{config::ExtractConfig, ExtractJob, ExtractJobKind, DEFAULT_REGEX, STEGO_FEATURE};

/// # 解压管理器
#[derive(Debug, Default)]
pub struct Extractor {
    config: ExtractConfig,
    files: Vec<PathBuf>,
    jobs: Vec<ExtractJob>,
}

impl Extractor {
    /// 解压管理器初始化
    pub fn load(&mut self, config: ExtractConfig) -> Result<()> {
        self.config = config;
        // 测试 7z
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
            Some(cap) => info!("7z ver.{}", &cap["version"]),
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
        info!("创建解压任务");
        self.load_unchecked_jobs();

        // 过滤 jobs 中文件数量不正确的任务
        self.jobs_filtrate_illegal_file_numeber();

        // 处理隐写
        if self.config.method.recogniz_steganography {
            // 尝试分离隐写文件
            self.separate_stego();
        };

        info!("载入解压任务完成，共 [ {} ] 个", self.jobs.len());
        debug!("解压任务列表：{:#?}", self.jobs);
    }

    fn load_unchecked_jobs(&mut self) {
        self.jobs = self
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

                    // 识别由 winrar 生成的 zip 分卷
                    let mut flag_zip_splited_by_winrar = false;
                    // 判断后缀名为 zip 且 文件头不是 &[0x50, 0x4B, 0x03, 0x04]
                    if item.extension().and_then(OsStr::to_str).unwrap_or("") == "zip" {
                        flag_zip_splited_by_winrar = match is_zip_splited_by_winrar(item) {
                            Ok(v) => v,
                            Err(e) => {
                                warn!("无法判断 zip 文件是否分卷[{}]: {}", item.display(), e);
                                false
                            }
                        };
                    };

                    if let Some(r) = matched_pattern {
                        // 如果与分卷文件名正则匹配
                        if let Some(cap) = r.captures(&filename) {
                            let package = cap["package"].to_string();
                            let vol = match cap["vol"].to_string().parse::<usize>() {
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
                                        "文件 \"{}\" 加入解压任务 [{}]",
                                        item.display(),
                                        value.package
                                    );
                                    if vol == 1 {
                                        // 如果是第 1 卷则加入解压路径
                                        value.path = item.to_path_buf();
                                    } else {
                                        // 如果不是第 1 卷则加入更新最大卷并加入关联列表
                                        match &mut value.kind {
                                            ExtractJobKind::Split {
                                                volume,
                                                zip_splited_by_winrar: _,
                                            } => {
                                                if &vol > volume {
                                                    *volume = vol
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
                                        "通过 \"{}\" 创建解压任务 [{}] ",
                                        item.display(),
                                        package
                                    );
                                    acc.insert(
                                        key.clone(),
                                        ExtractJob {
                                            package,
                                            kind: ExtractJobKind::Split {
                                                volume: vol,
                                                zip_splited_by_winrar: false,
                                            },
                                            path: match vol {
                                                1 => item.to_path_buf(),
                                                _ => PathBuf::new(),
                                            },
                                            token: String::new(),
                                            relevant: match vol {
                                                1 => vec![],
                                                _ => vec![item.to_path_buf()],
                                            },
                                        },
                                    );
                                }
                            };
                        }
                    } else {
                        // 如果与分卷文件名正则不匹配
                        let package =
                            match item.with_extension("").file_name().and_then(OsStr::to_str) {
                                Some(s) => s.to_string(),
                                None => {
                                    warn!("转换为任务失败：{}", item.display());
                                    return acc;
                                }
                            };

                        // key(是否分卷,目录,包名)
                        let key = (
                            flag_zip_splited_by_winrar,
                            dir_path.to_path_buf(),
                            package.clone(),
                        );

                        match flag_zip_splited_by_winrar {
                            // 若为特殊分卷
                            true => match acc.get_mut(&key) {
                                // 若列表中已有同键任务
                                Some(value) => {
                                    let vol = match value.kind {
                                        // 若原有任务应是分卷任务
                                        ExtractJobKind::Split {
                                            volume,
                                            zip_splited_by_winrar: _,
                                        } => Some(volume),
                                        // 若原有任务非分卷任务
                                        _ => {
                                            warn!(
                                                "解压任务 [{}] 不是分卷任务，文件未添加：{}",
                                                value.package,
                                                item.to_path_buf().display()
                                            );
                                            None
                                        }
                                    };
                                    if let Some(volume) = vol {
                                        value.kind = ExtractJobKind::Split {
                                            volume,
                                            zip_splited_by_winrar: true,
                                        };
                                        value.relevant.push(item.to_path_buf());
                                    };
                                }
                                // 若列表中没有同键任务
                                None => {
                                    acc.insert(
                                        key,
                                        ExtractJob {
                                            package,
                                            kind: ExtractJobKind::Split {
                                                volume: 0,
                                                zip_splited_by_winrar: true,
                                            },
                                            path: PathBuf::new(),
                                            token: String::new(),
                                            relevant: vec![item.to_path_buf()],
                                        },
                                    );
                                }
                            },
                            // 若为普通任务
                            false => {
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
                        };
                    }

                    acc
                },
            )
            .into_values()
            .collect();
    }

    fn separate_stego(&mut self) {
        // 除普通任务外不作处理
        let mut new_jobs = self
            .jobs
            .iter()
            .filter(|job| job.kind != ExtractJobKind::Normal)
            .map(Clone::clone)
            .collect::<Vec<ExtractJob>>();
        // 对不同任务尝试进行分离
        self.jobs
            .iter_mut()
            .filter(|job| job.kind == ExtractJobKind::Normal)
            .for_each(|job| match job.find_target_file_offset() {
                // 若取得偏移
                Ok(Some(offset)) => {
                    job.kind = ExtractJobKind::Stego(offset);
                    // 创建临时文件
                    info!("尝试分离隐写文件");
                    match job.create_temp_file() {
                        Ok(_) => {
                            info!("完成");
                            new_jobs.push(job.clone())
                        }
                        Err(e) => {
                            warn!("创建临时文件失败，跳过[{}]: {}", job.path.display(), e)
                        }
                    };
                }
                // 不符合隐写特征直接介入列表
                Ok(None) => {
                    debug!("没有找到隐写特征：{}", job.path.display());
                    new_jobs.push(job.clone())
                }
                // 识别出现错误则跳过
                Err(e) => {
                    warn!("分离隐写文件失败，跳过[{}]: {}", job.path.display(), e)
                }
            });
        self.jobs = new_jobs;
    }

    pub fn jobs_filtrate_illegal_file_numeber(&mut self) {
        let new_jobs = self
            .jobs
            .clone()
            .into_iter()
            .filter(ExtractJob::check_file_number)
            .collect();
        self.jobs = new_jobs;
    }
}

fn is_zip_splited_by_winrar(path: &Path) -> Result<bool> {
    // 读取 4B 数据
    let mut file_head: Vec<u8> = Vec::new();
    File::open(path)?.take(4).read_to_end(&mut file_head)?;
    Ok(file_head != STEGO_FEATURE[0].1)
}
