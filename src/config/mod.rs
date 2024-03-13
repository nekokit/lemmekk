//! # 程序配置模块

use std::{
    fs::{self},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    CliArgs, DeferOperation, ExtractMethod, MainCommand, TokenFilePattern, TokenListStyle,
    TokenProcess,
};

mod provider;
mod sample;

pub use provider::{COVER_FEATURE, DEFAULT_PATH, DEFAULT_REGEX, STEGO_FEATURE};

/// # 配置
/// 包括通用、解压和转换配置
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// 通用配置
    pub general: GeneralConfig,

    /// 解压配置
    pub extract: ExtractConfig,

    /// 密钥配置
    pub token: TokenConfig,
}

impl Config {
    /// 从配置文件载入
    ///
    /// # Arguments
    ///
    /// - `config_path` - 配置文件路径
    ///
    /// # Returns
    ///
    /// 文件中的配置
    pub fn load(config_path: &Path) -> Result<Config> {
        if !config_path.exists() {
            Self::create_sample(config_path).context("示例配置文件创建失败")?;
        };
        let mut config: Config = toml::from_str(&fs::read_to_string(config_path)?)?;
        debug!("读取配置文件\n{:#?}", config);
        // 如果配置文件为空字符串
        if config.general.token == PathBuf::new() {
            config.general.token = DEFAULT_PATH.token.to_path_buf();
            debug!(
                "general.token 为空字符串，使用默认路径：{}",
                config.general.token.display()
            );
        };
        if config.token.export_file == PathBuf::new() {
            config.token.export_file = DEFAULT_PATH.token_convert.to_path_buf();
            debug!(
                "token.export_file 为空字符串，使用默认路径：{}",
                config.token.export_file.display()
            );
        };
        if config.token.import_file == PathBuf::new() {
            config.token.import_file = DEFAULT_PATH.token_convert.to_path_buf();
            debug!(
                "token.log_file 为空字符串，使用默认路径：{}",
                config.token.import_file.display()
            );
        };
        if config.extract.path_7z == None || config.extract.path_7z == Some(PathBuf::new()) {
            config.extract.path_7z = None;
            debug!("extract.path_7z 为空字符串，将直接调用 7z");
        };
        Ok(config)
    }

    /// 在指定路径创建配置样板
    ///
    /// # Arguments
    ///
    /// - `path` - 文件路径
    fn create_sample(path: &Path) -> Result<()> {
        fs::File::create(path)?.write_all(sample::CONFIG.as_bytes())?;
        Ok(())
    }

    /// 使用 cli 参数覆盖配置
    ///
    /// # Arguments
    ///
    /// - `cli_args` - cli 参数
    pub fn overlay(mut self, cli_args: &CliArgs) -> Self {
        debug!("命令参数：\n{:#?}", cli_args);
        // General 表
        if let Some(path) = &cli_args.token {
            self.general.token = path.to_path_buf();
        };

        match &cli_args.main_command {
            // Token 表
            MainCommand::Token {
                command,
                add: _,
                delete: _,
            } => match command {
                Some(TokenProcess::List { style }) => {
                    if let Some(s) = style {
                        self.token.list_style = s.clone();
                    }
                }
                Some(TokenProcess::Export { pattern, file }) => {
                    if let Some(p) = pattern {
                        self.token.export_pattern = p.clone();
                    };
                    if let Some(f) = file {
                        self.token.export_file = f.clone();
                    };
                }
                Some(TokenProcess::Import { pattern, file }) => {
                    if let Some(p) = pattern {
                        self.token.import_pattern = p.clone();
                    };
                    if let Some(f) = file {
                        self.token.import_file = f.clone();
                    };
                }
                _ => {}
            },
            // Extract 表
            MainCommand::Extract {
                source,
                search_depth,
                excluded_suffix,
                tokens,
                token_hot_boundary,
                otutput_dir,
                defer_operation,
                recycle_dir,
                analyze_steganography,
                extract_directly,
                smart_directly,
                recursively,
            } => {
                if source.len() > 0 {
                    self.extract.source = source.clone();
                };
                if let Some(v) = search_depth {
                    self.extract.search_depth = *v;
                };
                if excluded_suffix.len() > 0 {
                    self.extract.excluded_suffix = excluded_suffix.clone();
                };
                if tokens.len() > 0 {
                    self.extract.tokens = tokens.clone();
                };
                if let Some(v) = token_hot_boundary {
                    self.extract.token_hot_boundary = *v;
                };
                if let Some(p) = otutput_dir {
                    self.extract.otutput_dir = p.to_path_buf();
                };
                if let Some(v) = defer_operation {
                    self.extract.defer_operation = v.clone();
                };
                if let Some(p) = recycle_dir {
                    self.extract.recycle_dir = p.to_path_buf();
                };
                if let Some(v) = analyze_steganography {
                    self.extract.method.analyze_steganography = *v;
                };
                if let Some(v) = extract_directly {
                    self.extract.method.extract_directly = *v;
                };
                if let Some(v) = smart_directly {
                    self.extract.method.smart_directly = *v;
                };
                if let Some(v) = recursively {
                    self.extract.method.recursively = *v;
                };
            }
        }
        self
    }
}

/// # 通用配置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// 密钥文件
    pub token: PathBuf,
}
impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            token: DEFAULT_PATH.token.to_path_buf(),
        }
    }
}

/// # 密钥配置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenConfig {
    /// 密钥列出模式
    pub list_style: TokenListStyle,
    /// 导出模式
    pub export_pattern: TokenFilePattern,
    /// 导出文件
    pub export_file: PathBuf,
    /// 导入模式
    pub import_pattern: TokenFilePattern,
    /// 导入文件
    pub import_file: PathBuf,
}
impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            list_style: TokenListStyle::Plain,
            import_pattern: TokenFilePattern::Plain,
            import_file: DEFAULT_PATH.token_convert.to_path_buf(),
            export_pattern: TokenFilePattern::Plain,
            export_file: DEFAULT_PATH.token_convert.to_path_buf(),
        }
    }
}

/// # 解压配置
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtractConfig {
    /// 7zip 程序路径
    pub path_7z: Option<PathBuf>,
    /// 需解压的文件或目录
    pub source: Vec<PathBuf>,
    /// 文件夹搜索深度
    pub search_depth: i8,
    /// 排除的文件扩展名
    pub excluded_suffix: Vec<String>,
    /// 优先使用的密钥
    pub tokens: Vec<String>,
    /// 常用密钥存留时间
    pub token_hot_boundary: usize,
    /// 解压目标文件夹
    pub otutput_dir: PathBuf,
    /// 解压后对压缩文件的操作
    pub defer_operation: DeferOperation,
    /// 回收文件夹
    pub recycle_dir: PathBuf,
    /// 回收文件夹
    pub method: ExtractMethod,
}
impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            path_7z: None,
            source: vec![],
            search_depth: 0,
            excluded_suffix: vec![],
            tokens: vec![],
            token_hot_boundary: 30,
            otutput_dir: PathBuf::new(),
            defer_operation: DeferOperation::DoNothing,
            recycle_dir: PathBuf::new(),
            method: ExtractMethod::default(),
        }
    }
}
impl ExtractConfig {
    pub fn get_command_7z(&self) -> PathBuf {
        match &self.path_7z {
            Some(p) => p.to_path_buf(),
            None => PathBuf::from("7z"),
        }
    }
}
