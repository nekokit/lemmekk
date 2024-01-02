//! # 程序配置
//! 提供默认配置、新建配置文件和从配置文件读取配置。

use std::{
    fmt::Display,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process, vec,
};

use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use colored::Colorize;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    app::{CliArgs, Command, PasswordProcess},
    log::LogLevel,
    PasswordFile, DEFAULT_PATH, DEFAULT_REGEX,
};

mod sample;

/// # 配置结构体
/// 保存所有配置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// 通用配置
    pub general: GeneralConfig,

    /// 解压配置
    pub extract: ExtractConfig,

    /// 转换配置
    pub convert: ConvertConfig,
}

impl Config {
    /// 检查配置文件路径，若文件不存在则创建示例配置文件
    ///
    /// # Arguments
    ///
    /// - `path` - 文件路径
    ///
    pub fn check_path(path: &Path) -> Result<()> {
        if !path.exists() {
            Self::create_sample(path).context("示例配置文件写入失败")?;
        } else if !path.is_file() {
            bail!("配置文件路径有误: '{}'", path.display());
        }
        Ok(())
    }

    /// 在指定路径创建配置样板
    ///
    /// # Arguments
    ///
    /// - `path` - 文件路径
    ///
    fn create_sample(path: &Path) -> Result<()> {
        fs::File::create(path)?.write_all(sample::CONFIG.as_bytes())?;
        Ok(())
    }

    /// 从文件载入配置文件
    ///
    /// # Arguments
    ///
    /// - `path` - 文件路径
    ///
    /// # Returns
    ///
    /// 文件中的配置
    ///
    pub fn load(path: &Path) -> Result<Self> {
        let config: Config = toml::from_str(&fs::read_to_string(path)?)?;
        Ok(config)
    }

    /// 使用 cli 参数覆盖配置
    ///
    /// # Arguments
    ///
    /// - `args` - cli 参数
    ///
    pub fn overlay(mut self, args: &CliArgs) -> Self {
        if let Some(v) = &args.log_level {
            self.general.log_level = v.clone();
        };
        if let Some(v) = &args.log_file {
            self.general.log_path = v.clone();
        };
        if let Some(v) = &args.password_file {
            self.general.password_path = v.clone();
        };
        match &args.command {
            Command::Extract {
                path_for_7z,
                extract_input,
                walk_input,
                excluded_extension,
                extract_output,
                passwords,
                operation_for_extracted,
                dir_for_move,
                recogniz_steganography,
                extract_directly,
                extract_directly_single,
                recursively,
            } => {
                if let Some(v) = path_for_7z {
                    self.extract.path_for_7z = Some(v.clone());
                };
                if extract_input.len() > 0 {
                    self.extract.extract_input = extract_input.clone();
                };
                if let Some(v) = walk_input {
                    self.extract.walk_input = *v;
                };
                if excluded_extension.len() > 0 {
                    self.extract.excluded_extension = excluded_extension.clone();
                };
                if let Some(v) = extract_output {
                    self.extract.extract_output = v.clone();
                };
                if passwords.len() > 0 {
                    self.extract.passwords = passwords.clone();
                };
                if let Some(v) = operation_for_extracted {
                    self.extract.extract_method.operation_for_extracted = v.clone();
                };
                if let Some(v) = dir_for_move {
                    self.extract.extract_method.dir_for_move = v.clone();
                };
                if let Some(v) = recogniz_steganography {
                    self.extract.extract_method.recogniz_steganography = *v;
                };
                if let Some(v) = extract_directly {
                    self.extract.extract_method.extract_directly = *v;
                };
                if let Some(v) = extract_directly_single {
                    self.extract.extract_method.extract_directly_single = *v;
                };
                if let Some(v) = recursively {
                    self.extract.extract_method.recursively = *v;
                };
            }
            Command::Password {
                command,
                add: _,
                del: _,
            } => {
                if let Some(cmd) = command {
                    match cmd {
                        PasswordProcess::List => {}
                        PasswordProcess::Export {
                            export_type,
                            export_path,
                        } => {
                            if let Some(v) = export_type {
                                self.convert.export_type = v.clone();
                            };
                            if let Some(v) = export_path {
                                self.convert.export_path = v.clone();
                            };
                        }
                        PasswordProcess::Import {
                            import_type,
                            import_path,
                        } => {
                            if let Some(v) = import_type {
                                self.convert.import_type = v.clone();
                            };
                            if let Some(v) = import_path {
                                self.convert.import_path = v.clone();
                            };
                        }
                    }
                }
            }
        }

        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            extract: ExtractConfig::default(),
            convert: ConvertConfig::default(),
        }
    }
}

/// # 基本设置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// 日志等级
    pub log_level: LogLevel,
    /// 日志文件
    pub log_path: PathBuf,
    /// 密码文件
    pub password_path: PathBuf,
}
impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            log_path: DEFAULT_PATH.log.clone(),
            password_path: DEFAULT_PATH.password.clone(),
        }
    }
}

impl GeneralConfig {
    /// 检查 general 配置段，若配置中指定文件不存在则创建
    pub fn check(&self) -> Result<()> {
        if !self.log_path.exists() || self.log_path.is_file() {
            fs::File::create(&self.log_path).context("日志文件创建失败")?;
            info!("使用日志文件: '{}'", self.log_path.display());
        } else {
            let msg = format!(
                "日志文件路径有误: '{}'",
                self.log_path.display().to_string().yellow()
            );
            warn!("{}", msg);
            bail!(msg);
        };

        if !self.password_path.exists() {
            warn!(
                "未找到密码文件: '{}'",
                self.password_path.display().to_string().yellow()
            );
            info!("创建密码文件示例: '{}'", self.password_path.display());
            PasswordFile::write_sample(&self.password_path).context("示例密码文件创建失败")?;
        } else if !self.password_path.is_file() {
            let msg = format!(
                "密码文件路径有误: '{}'",
                self.password_path.display().to_string().yellow()
            );
            warn!("{}", msg);
            bail!(msg);
        };

        Ok(())
    }
}

/// # 解压设置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtractConfig {
    /// 7zip 路径
    pub path_for_7z: Option<PathBuf>,
    /// 待解压文件或文件夹
    pub extract_input: Vec<PathBuf>,
    /// 搜索子文件夹
    pub walk_input: bool,
    /// 搜索子文件夹
    pub excluded_extension: Vec<String>,
    /// 解压目的文件夹
    pub extract_output: PathBuf,
    /// 指定密码
    pub passwords: Vec<String>,
    /// 常用密码存留时间
    pub password_hot_boundary: u32,
    /// 解压方式
    pub extract_method: ExtractMethod,
}
impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            path_for_7z: None,
            extract_input: vec![],
            walk_input: false,
            excluded_extension: vec![],
            extract_output: PathBuf::new(),
            passwords: vec![],
            password_hot_boundary: 30,
            extract_method: ExtractMethod::default(),
        }
    }
}

impl ExtractConfig {
    /// 检查 extract 配置段，若配置中指定文件不存在则创建
    pub fn check(&self) -> Result<()> {
        // 检查 7z
        let cmd_7z: String;
        if let Some(path) = &self.path_for_7z {
            cmd_7z = path.display().to_string();
        } else {
            cmd_7z = "7z".to_string();
        }
        debug!("7z 调用: {}", cmd_7z);
        let version = Self::check_7z(&cmd_7z).context("7zip 执行文件检查失败")?;
        info!("7z 版本: {}", version);

        // 检查解压目标文件夹
        if !self.extract_output.exists() {
            fs::create_dir(&self.extract_output).context("解压目标文件夹创建失败")?;
        } else if !self.extract_output.is_dir() {
            let msg = format!(
                "解压文件夹有误: '{}'",
                self.extract_output.display().to_string().red()
            );
            error!("{}", msg);
            bail!(msg);
        }

        // 解压后移动时检查移动文件夹
        if let OperationForExtracted::Move = self.extract_method.operation_for_extracted {
            if !self.extract_method.dir_for_move.exists() {
                fs::create_dir(&self.extract_method.dir_for_move)
                    .context("解压后移动文件夹创建失败")?;
            } else if !self.extract_method.dir_for_move.is_dir() {
                let msg = format!(
                    "解压后移动文件夹有误: '{}'",
                    self.extract_output.display().to_string().red()
                );
                error!("{}", msg);
                bail!(msg);
            }
        }

        Ok(())
    }

    fn check_7z(path_str: &str) -> Result<String> {
        let output = process::Command::new(path_str)
            .arg("--help")
            .output()
            .context(format!("7z 路径有误: {}", path_str.to_string().red()))?;

        let version = match DEFAULT_REGEX
            .version_7z
            .captures(&String::from_utf8(output.stdout)?)
        {
            None => {
                let msg = format!("未知的 7z 版本: {}", path_str.to_string().red());
                error!("{}", msg);
                bail!(msg);
            }
            Some(v) => v[1].to_string(),
        };
        Ok(version)
    }
}

/// # 解压选项
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtractMethod {
    /// 解压后操作
    pub operation_for_extracted: OperationForExtracted,
    /// 移动目的文件夹
    pub dir_for_move: PathBuf,
    /// 移动目的文件夹
    pub recogniz_steganography: bool,
    /// 是否不为每个压缩包创建目录，直接解压到目标文件夹
    pub extract_directly: bool,
    /// 在压缩包内只有单文件单文件夹的情况下，是否不创建文件夹直接解压
    pub extract_directly_single: bool,
    /// 是否递归解压压缩文件内的压缩文件
    pub recursively: bool,
}
impl Default for ExtractMethod {
    fn default() -> Self {
        Self {
            operation_for_extracted: OperationForExtracted::DoNothing,
            dir_for_move: PathBuf::new(),
            recogniz_steganography: false,
            extract_directly: false,
            extract_directly_single: false,
            recursively: false,
        }
    }
}

/// # 导入导出设置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ConvertConfig {
    /// 导入格式
    pub import_type: PasswordConvertType,
    /// 导入文件
    pub import_path: PathBuf,
    /// 导出格式
    pub export_type: PasswordConvertType,
    /// 导出文件
    pub export_path: PathBuf,
}
impl Default for ConvertConfig {
    fn default() -> Self {
        Self {
            import_type: PasswordConvertType::Text,
            import_path: DEFAULT_PATH.convert.clone(),
            export_type: PasswordConvertType::Text,
            export_path: DEFAULT_PATH.convert.clone(),
        }
    }
}

/// # 解压后操作枚举
#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum OperationForExtracted {
    /// 不做任何处理
    #[value(help = "不做任何处理")]
    DoNothing,
    /// 删除压缩文件
    #[value(help = "删除压缩文件")]
    Delete,
    /// 移动压缩文件
    #[value(help = "移动压缩文件")]
    Move,
}

/// # 密码导入导出类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum PasswordConvertType {
    /// 普通格式，密码一行一个，格式：`{密码}`
    #[value(help = "普通格式")]
    Text,

    /// 解TMD压格式，密码一行一个，格式：`{密码}\t\t{使用次数}` 。
    #[value(help = "普通格式")]
    Jtmdy,
}
impl Display for PasswordConvertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordConvertType::Text => write!(f, "Text"),
            PasswordConvertType::Jtmdy => write!(f, "Jtmdy"),
        }
    }
}
