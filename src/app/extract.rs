//! # 解压模块

use log::{debug, info, trace, warn};
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{Duration, Utc};
use colored::Colorize;
use walkdir::WalkDir;

use crate::config::ExtractConfig;
use crate::{filepattern, CompressType, ImageType, PasswordFile, PasswordList};

/// # 解压流程
pub struct ExtractProtocol<'a> {
    /// 密码列表
    pub passwords: PasswordList<'a>,
    /// 待解压列表
    pub pending_jobs: Vec<ExtractJob>,
}

impl<'a> Default for ExtractProtocol<'a> {
    fn default() -> Self {
        Self {
            passwords: PasswordList {
                runtime: vec![],
                frequent: vec![],
                others: vec![],
            },
            pending_jobs: vec![],
        }
    }
}

impl<'a> ExtractProtocol<'a> {
    /// 根据指定密码、密码热边界分类加载密码
    ///
    /// # Arguments
    ///
    /// - 'password_file' - 读取的密码文件
    /// - 'extract_config' - 解压配置
    ///
    pub fn load_passwords(
        &mut self,
        password_file: &'a mut PasswordFile,
        extract_config: &ExtractConfig,
    ) {
        let hot_boundary = Utc::now() - Duration::days(extract_config.password_hot_boundary.into());
        password_file.passwords.iter_mut().for_each(|p| {
            if extract_config.passwords.contains(&p.password) {
                // 优先密码储存于运行级别
                self.passwords.runtime.push(p);
            } else if p.gmt_usage > hot_boundary {
                // 使用时间大于热边界的储存于频繁级别
                self.passwords.frequent.push(p);
            } else {
                // 其他的储存于低优先级
                self.passwords.others.push(p);
            }
        });
        debug!("使用密码: {:?}", self.passwords);
    }

    /// 从文件列表尝试加载解压任务
    ///
    /// # Arguments
    ///
    /// - 'paths' - 路径列表
    /// - 'walkdir' - 是否解压子文件夹中的文件
    ///
    /// # Returns
    ///
    /// 转换成功的数量
    ///
    pub fn load_jobs(&mut self, paths: &[PathBuf], walkdir: bool) -> Result<usize> {
        let path_list = Self::collect_file(paths, walkdir);
        self.pending_jobs = path_list
            .iter()
            .map(Self::convert_to_job) // 转为解压任务
            .filter_map(Result::ok) // 筛选出转换成功的
            .collect();
        Ok(self.pending_jobs.len())
    }

    /// 从路径读取文件尝试转为解压任务
    ///
    /// # Arguments
    ///
    /// - 'path' - 路径列表
    ///
    /// # Returns
    ///
    /// 转换结果
    ///
    fn convert_to_job(path: &PathBuf) -> Result<ExtractJob> {
        // 读取 8 字节匹配压缩文件头
        let mut file_head = Vec::new();
        File::open(path)?.take(8).read_to_end(&mut file_head)?;
        if let Ok(t) = filepattern::match_compressed_header(&file_head) {
            // 匹配成功则组装解压任务
            info!(
                "已添加 [{}]: '{}'",
                "压缩文件".to_string().blue(),
                path.display()
            );
            return Ok(ExtractJob {
                path: path.to_path_buf(),
                compress_type: t,
                relevant_file: RelevantFile::None,
            });
        } else {
            // 压缩文件匹配失败则尝试匹配图片头
            let job = ImageCover {
                path: path.to_path_buf(),
                image_type: filepattern::match_image_header(&file_head)?,
            };

            // 图片头匹配成功则尝试搜索压缩文件偏移
            debug!(
                "尝试分离 [{}]: '{}'",
                "隐写文件".to_string().blue(),
                path.display()
            );
            match job.generate_job_file() {
                Ok(job) => {
                    info!(
                        "已添加 [{}]: '{}'",
                        "隐写文件".to_string().blue(),
                        job.path.display()
                    );
                    Ok(job)
                }
                Err(e) => {
                    info!(
                        "忽略 [{}]: '{}'",
                        "隐写文件".to_string().blue(),
                        job.path.display(),
                    );
                    Err(e)
                }
            }
        }
    }

    /// 搜集输入目录中的待解压文件
    ///
    /// # Arguments
    ///
    /// - 'paths' - 路径列表
    /// - 'walkdir' - 是否解压子文件夹中的文件
    ///
    /// # Returns
    ///
    /// 所有文件列表
    ///
    fn collect_file(paths: &[PathBuf], walkdir: bool) -> Vec<PathBuf> {
        let mut list = vec![];
        paths.iter().for_each(|path| {
            if path.is_file() {
                // 路径为文件直接添加
                list.push(path.clone());
            } else if path.is_dir() && walkdir {
                // 路径为文件夹且需要遍历子文件夹
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|item| item.path().is_file())
                    .for_each(|item| {
                        let path = item.path().to_path_buf();
                        trace!("找到文件: '{}'", path.display());
                        list.push(path)
                    });
            } else if path.is_dir() && !walkdir {
                // 路径为文件夹且不遍历子文件夹
                path.read_dir()
                    .unwrap()
                    .filter_map(Result::ok)
                    .filter(|item| item.path().is_file())
                    .for_each(|item| {
                        let path = item.path().to_path_buf();
                        trace!("找到文件: '{}'", path.display());
                        list.push(path)
                    });
            } else {
                // 错误路径丢弃
                warn!(
                    "指定的解压文件(夹)有误: '{}'",
                    path.display().to_string().yellow()
                )
            }
        });
        list
    }
}

/// # 解压任务
pub struct ExtractJob {
    /// 压缩文件路径
    pub path: PathBuf,
    /// 压缩格式
    pub compress_type: CompressType,
    /// 相关文件
    relevant_file: RelevantFile,
}

/// # 解压任务关联文件
///
/// 用于解压完成后一并操作
enum RelevantFile {
    /// 无关联文件
    None,
    /// 有其他分卷
    Split(Vec<PathBuf>),
    /// 有隐写文件
    Steganography([PathBuf; 2]),
}

/// # 图片隐写分离任务
struct ImageCover {
    /// 文件路径
    pub path: PathBuf,
    /// 图片格式
    pub image_type: ImageType,
}

impl ImageCover {
    /// 尝试分离隐写文件并生成解压任务
    ///
    /// # Returns
    ///
    /// 转换的解压任务
    ///
    fn generate_job_file(&self) -> Result<ExtractJob> {
        // 取得压缩文件偏移和压缩文件类型
        let (index, compress_type) = self
            .calculate_compressed_file_index()
            .context("压缩文件特征不存在")?;
        trace!("取得压缩文件偏移: '{}'", index.to_string().green());

        // 生成临时文件的文件名
        let img_path = self.path.with_file_name(
            self.path
                .file_name()
                .ok_or_else(|| anyhow!("文件名非法: '{}'", self.path.display()))?
                .to_string_lossy()
                .to_string()
                + ".img.temp",
        );
        let file_path = self.path.with_file_name(
            self.path
                .file_name()
                .ok_or_else(|| anyhow!("文件名非法: '{}'", self.path.display()))?
                .to_string_lossy()
                .to_string()
                + ".file.temp",
        );

        // 分别准备图片与压缩文件的数据
        let mut img = vec![0; index];
        let mut reader = io::BufReader::new(File::open(&self.path)?);
        reader.read_exact(&mut img)?;
        if img_path.exists() {
            // 如果存在同名文件则跳过
            let msg = format!(
                "已存在文件: '{}', 跳过隐写文件: '{}'",
                img_path.display(),
                &self.path.display(),
            );
            info!("{}", msg);
            bail!(msg);
        } else if file_path.exists() {
            let msg = format!(
                "已存在文件: '{}', 跳过隐写文件: '{}'",
                img_path.display(),
                &self.path.display()
            );
            info!("{}", msg);
            bail!(msg);
        } else {
            // 如果文件不存在则写入
            debug!("分离图片文件: {}'", img_path.display());
            File::create(&img_path)?.write_all(&img)?;

            debug!("分离压缩文件: {}'", file_path.display());
            let mut writer = BufWriter::new(File::create(&file_path)?);
            io::copy(&mut reader, &mut writer)?;
        };

        Ok(ExtractJob {
            path: file_path,
            compress_type,
            relevant_file: RelevantFile::Steganography([self.path.to_path_buf(), img_path]),
        })
    }

    /// 计算压缩文件的偏移和压缩文件类型
    ///
    /// # Returns
    ///
    /// - 压缩文件的偏移
    /// - 压缩文件类型
    ///
    fn calculate_compressed_file_index(&self) -> Result<(usize, CompressType)> {
        // 读取 8MB 数据
        let mut file_head = Vec::new();
        File::open(&self.path)?
            .take(8 << 20)
            .read_to_end(&mut file_head)?;

        // 查询图片文件特征
        let image_fearure = filepattern::get_image_feature(self.image_type)?;
        // 取得压缩文件偏移
        let index = file_head
            .windows(image_fearure.2.len()) // 使用文件尾长度的窗口
            .position(|s| s == image_fearure.2) // 匹配每个窗口尝试取得图片尾索引
            .context("图片结束标记未在前 8M 数据中找到")?
            + image_fearure.2.len(); // 加上文件尾大小取得压缩文件偏移

        // 在偏移量后方取 8 字节重新定义文件头以匹配压缩文件头
        let file_head = if index >= file_head.len() - 8 {
            // 如果不足 8 字节则放弃
            let msg = format!("压缩文件特征不存在: '{}'", self.path.display());
            info!("{}", msg);
            bail!(msg);
        } else {
            &file_head[index..index + 8]
        };
        let compress_type = filepattern::match_compressed_header(&file_head)?;

        Ok((index, compress_type))
    }
}
