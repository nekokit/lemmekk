//! # 程序配置模块

use std::{
    fs::{self},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{CliArgs, MainCommand, TokenFilePattern, TokenListStyle, TokenProcess};

mod provider;
mod sample;

pub use provider::{DEFAULT_PATH, DEFAULT_REGEX};

/// # 配置
/// 包括通用、解压和转换配置
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// 通用配置
    pub general: GeneralConfig,

    /// 解压配置
    // pub extract: ExtractConfig,

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
            // ...
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
