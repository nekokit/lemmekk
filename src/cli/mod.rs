//! # CLI 模块

use std::fs;

use anyhow::{Context, Ok, Result};
use log::{debug, info};

use crate::{Config, TokenManager, DEFAULT_PATH};

mod args;

pub use args::{CliArgs, MainCommand, TokenProcess};

/// # 应用程序对象
#[derive(Debug)]
pub struct Cli {
    /// 参数模块
    pub cli_args: CliArgs,
    /// 配置模块
    pub config: Config,
    /// 密钥模块
    pub token_manager: TokenManager,
}

impl Cli {
    /// 通过程序参数创建 App 对象
    ///
    /// # Arguments
    ///
    /// - `cli_args` - clap 程序参数对象
    ///
    /// # Returns
    ///
    /// Cli 对象
    pub fn create(cli_args: CliArgs) -> Result<Self> {
        // 优先从程序参数获取配置文件路径，否则使用默认
        let config_path = match &cli_args.config {
            Some(p) => p.to_path_buf(),
            None => {
                if !DEFAULT_PATH.data_dir.exists() {
                    fs::create_dir(&DEFAULT_PATH.data_dir)?;
                };
                DEFAULT_PATH.config.to_path_buf()
            }
        };

        debug!("配置文件路径：{}", config_path.display());
        let config = Config::load(&config_path)
            .context("配置文件加载失败")?
            .overlay(&cli_args);

        let token_manager = TokenManager::default();
        // let extractor = Extractor::default();

        Ok(Self {
            cli_args,
            config,
            token_manager,
        })
    }

    /// 程序执行入口
    pub fn startup(&mut self) -> Result<()> {
        debug!("已加载配置：\n{:#?}", self.config);
        self.token_manager
            .load(&self.config.general.token)
            .context("密钥文件加载失败")?;
        match &self.cli_args.main_command {
            // 主命令：密钥操作
            MainCommand::Token {
                command,
                add,
                delete,
            } => match command {
                // 增添密钥
                None => {
                    let mut flag_is_updated = false;
                    if delete.len() > 0 {
                        let delete_count = self.token_manager.delete_tokens(delete);
                        info!("删除密钥：{:?} 共删除 {} 个", delete, delete_count);
                        println!("删除密码 {} 个", delete_count);
                        flag_is_updated = true;
                    }
                    if add.len() > 0 {
                        let add_count = self.token_manager.add_tokens(add);
                        info!("添加密钥：{:?} 共添加 {} 个", add, add_count);
                        println!("添加密码 {} 个", add_count);
                        flag_is_updated = true;
                    }
                    if flag_is_updated {
                        self.token_manager.write()?;
                    }
                }

                // 列出密钥
                Some(TokenProcess::List { style: _ }) => {
                    println!(
                        "{}",
                        self.token_manager.display(&self.config.token.list_style)
                    );
                }

                // 导出密钥
                Some(TokenProcess::Export {
                    pattern: _,
                    file: _,
                }) => {}

                // 导入密钥
                Some(TokenProcess::Import {
                    pattern: _,
                    file: _,
                }) => {}
            },
        }
        Ok(())
    }
}
