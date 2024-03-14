//! # 解压任务模块

use std::{
    ffi::OsStr,
    fmt::Display,
    fs::File,
    io::{self, BufWriter, Read, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use log::{debug, info};

use crate::{COVER_FEATURE, STEGO_FEATURE};

/// # 解压任务
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExtractJob {
    pub package: String,
    pub kind: ExtractJobKind,
    pub path: PathBuf,
    pub token: String,
    pub relevant: Vec<PathBuf>,
}
impl Display for ExtractJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {{ {:?} | {} | `{}` | {:?} }}",
            self.package,
            self.kind,
            self.path.display(),
            self.token,
            self.relevant,
        )
    }
}

impl ExtractJob {
    /// 搜索隐写文件中的压缩文件偏移
    pub fn find_target_file_offset(&self) -> Result<Option<usize>> {
        if let ExtractJobKind::Stego(offset) = self.kind {
            return Ok(Some(offset));
        }
        // 读取 8MB 数据
        let mut file_head: Vec<u8> = Vec::new();
        File::open(&self.path)?
            .take(8 << 20)
            .read_to_end(&mut file_head)?;
        let feat = COVER_FEATURE.iter().fold(None, |acc, feat| {
            if file_head.starts_with(feat.1) {
                Some(feat.clone())
            } else {
                acc
            }
        });
        match feat {
            Some((kind, _, tail)) => {
                info!("发现隐写文件[{}]: {}", kind, self.path.display());

                match file_head.windows(tail.len()).position(|s| s == tail) {
                    // 8M 范围内匹配伪装文件尾和压缩文件特征
                    Some(value) => {
                        let offset_1 = value + tail.len();
                        let mut offset_2 = 0;

                        // 在之后的数据中匹配压缩文件特征
                        let stego_type = STEGO_FEATURE.iter().find(|&&(_, head)| match &file_head
                            [offset_1..]
                            .windows(head.len())
                            .position(|s| s == head)
                        {
                            Some(offset) => {
                                offset_2 = *offset;
                                true
                            }
                            None => false,
                        });

                        match stego_type {
                            // 有压缩文件的特征
                            Some(&(t, _)) => {
                                let offset = offset_1 + offset_2;
                                info!("已取得偏移 [ {} | {} ]", t, offset);
                                Ok(Some(offset_1))
                            }
                            // 无压缩文件特征
                            None => bail!("8M 范围内找不到压缩文件特征"),
                        }
                    }
                    // 找不到伪装文件尾返回Err
                    None => bail!("8M 范围内找不到压缩文件特征"),
                }
            }
            // 不匹配隐写特征则返回 None
            None => Ok(None),
        }
    }

    /// 根据偏移创建临时文件
    pub fn create_temp_file(&mut self) -> Result<()> {
        if let ExtractJobKind::Stego(offset) = self.kind {
            if offset == 0 {
                return Ok(());
            };
            let file_name = match self.path.file_name().and_then(OsStr::to_str) {
                Some(v) => v,
                None => {
                    bail!("获取文件名失败：{}", self.path.display())
                }
            };
            let temp_basis_path = self
                .path
                .with_file_name("[lemmkk]".to_string() + file_name)
                .with_extension("basis");
            let temp_file_path = self
                .path
                .with_file_name("[lemmkk]".to_string() + file_name)
                .with_extension("file");

            // 分别准备伪装文件与压缩文件的数据
            let mut basis = vec![0; offset];
            let mut reader = io::BufReader::new(File::open(&self.path)?);
            reader.read_exact(&mut basis)?;

            if !temp_basis_path.exists() && !temp_file_path.exists() {
                // 如果文件不存在则写入
                debug!("分离伪装文件: {}'", temp_basis_path.display());
                File::create(&temp_basis_path)?.write_all(&basis)?;

                debug!("分离压缩文件: {}'", temp_file_path.display());
                let mut writer = BufWriter::new(File::create(&temp_file_path)?);
                io::copy(&mut reader, &mut writer)?;
            } else {
                // 如果文件存在则放弃
                bail!("临时文件重名");
            }

            self.relevant.push(temp_basis_path.to_path_buf());
            self.relevant.push(self.path.to_path_buf());
            self.path = temp_file_path;
        }
        Ok(())
    }
}

/// # 解压任务类型
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtractJobKind {
    Split(usize),
    Stego(usize),
    Normal,
}
impl Default for ExtractJobKind {
    fn default() -> Self {
        Self::Normal
    }
}
