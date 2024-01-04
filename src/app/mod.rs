//! # 应用程序

use anyhow::{Context, Result};
use colored::Colorize;
use log::{debug, info};

use crate::{config::Config, PasswordManager, DEFAULT_PATH};

mod args;
mod extract;

pub use args::CliArgs;
pub use args::Command;
pub use args::PasswordProcess;
pub use extract::Extractor;

/// # 应用程序对象
pub struct Application<'a> {
    /// 来自 cli 的参数
    args: CliArgs,
    /// 执行的配置
    pub config: Config,
    /// 解压流程
    pub extract: Extractor,
    /// 密码模块
    pub password: PasswordManager<'a>,
}

impl<'a> Application<'a> {
    /// 通过程序参数创建 App 对象
    ///
    /// # Arguments
    ///
    /// - `args` - clap 程序参数对象
    ///
    /// # Returns
    ///
    /// App 对象
    pub fn create(args: CliArgs) -> Result<Self> {
        // 优先从程序参数获取配置文件路径，否则使用默认
        let path = match &args.config_file {
            Some(p) => p.clone(),
            None => DEFAULT_PATH.config.clone(),
        };
        let config = Config::load(&path)
            .context("配置文件加载失败")?
            .overlay(&args);
        let password = PasswordManager::load(&config.general.password_path)?;
        let extract = Extractor::default();

        Ok(Self {
            args,
            config,
            extract,
            password,
        })
    }

    /// 主程序运行逻辑
    pub fn run(&'a mut self) -> Result<()> {
        debug!("命令行参数: {:?}", self.args);
        debug!("配置: {:?}", self.config);

        self.config
            .general
            .check()
            .context("配置文件 [general] 检查失败")?;
        self.password = PasswordManager::load(&self.config.general.password_path)
            .context("密码文件加载失败")?;
        info!(
            "已读取密码 {} 个",
            self.password.count().to_string().bold().green()
        );

        match &self.args.command {
            Command::Extract {
                path_for_7z: _,
                extract_input: _,
                walk_input: _,
                excluded_extension: _,
                extract_output: _,
                passwords: _,
                operation_for_extracted: _,
                dir_for_move: _,
                recogniz_steganography: _,
                extract_directly: _,
                extract_directly_single: _,
                recursively: _,
            } => {
                // 解压操作
                self.config
                    .extract
                    .check()
                    .context("配置文件 [extract] 检查失败")?;
                self.extract.run(&mut self.password, &self.config.extract)?;
            }
            Command::Password { command, add, del } => {
                // 密码操作
                self.password.run(command, add, del, &self.config.convert)?;
            }
        }
        Ok(())
    }
}
