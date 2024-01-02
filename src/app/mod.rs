//! # 应用程序
use std::fs;

use anyhow::{Context, Result};
use colored::Colorize;
use log::{debug, info};

use crate::{config::Config, PasswordFile, DEFAULT_PATH};

mod args;
mod extract;

pub use args::CliArgs;
pub use args::Command;
pub use args::PasswordProcess;
pub use extract::ExtractProtocol;

/// # 应用程序对象
pub struct Application<'a> {
    /// 来自 cli 的参数
    args: CliArgs,
    /// 执行的配置
    pub config: Config,
    /// 解压流程
    pub extract: ExtractProtocol<'a>,
    /// 密码文件
    passwords_file: PasswordFile,
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
        Config::check_path(&path).context("配置文件路径检查失败")?;
        let config = Config::load(&path)
            .context("配置文件加载失败")?
            .overlay(&args);

        Ok(Self {
            args,
            config,
            extract: ExtractProtocol::default(),
            passwords_file: PasswordFile::default(),
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
        self.passwords_file =
            PasswordFile::load(&self.config.general.password_path).context("密码文件读取失败")?;
        info!(
            "已读取密码 {} 个",
            self.passwords_file.count().to_string().bold().green()
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

                // 添加指定密码并写入文件
                if self.config.extract.passwords.len() > 0 {
                    let count = self
                        .passwords_file
                        .add_passwords(&self.config.extract.passwords);
                    self.passwords_file
                        .sort_and_write(&self.config.general.password_path)
                        .context("密码文件写入失败")?;
                    info!(
                        "已添加指定的密码，共 {} 个",
                        count.to_string().bold().green()
                    );
                }

                // 检查配置
                self.config
                    .extract
                    .check()
                    .context("配置文件 [extract] 检查失败")?;

                // 加载密码
                self.extract
                    .load_passwords(&mut self.passwords_file, &self.config.extract);

                // 加载待解压任务
                let job_count = self
                    .extract
                    .load_jobs(&self.config.extract.extract_input, &self.config.extract)?;
                info!("共加载 {} 个解压任务", job_count.to_string().green());
            }
            Command::Password { command, add, del } => {
                // 密码操作
                if let Some(command) = command {
                    match command {
                        PasswordProcess::List => {
                            // 列出密码
                            println!("{}", self.passwords_file.display())
                        }
                        PasswordProcess::Export {
                            export_type: _,
                            export_path: _,
                        } => {
                            // 导出密码
                            let password_type = &self.config.convert.export_type;
                            let path = &self.config.convert.export_path;
                            if !path.exists() {
                                fs::File::create(&path)
                                    .context(format!("导出文件创建失败: '{}'", &path.display()))?;
                            }

                            info!(
                                "导出密码[{}]: {}",
                                password_type.to_string().blue(),
                                path.display()
                            );
                            let count = self
                                .passwords_file
                                .export(path, password_type)
                                .context("密码导出失败")?;
                            let msg = format!("已导出 {} 个密码", count.to_string().bold().green());
                            println!("{}", msg);
                            info!("{}", msg)
                        }
                        PasswordProcess::Import {
                            import_type,
                            import_path,
                        } => {
                            // 导入密码
                            if let Some(t) = import_type {
                                self.config.convert.import_type = t.clone();
                            }
                            if let Some(p) = import_path {
                                self.config.convert.import_path = p.clone();
                            }
                            let p = &self.config.convert.import_path;
                            let t = &self.config.convert.import_type;
                            info!("导入密码[{}]: {}", t.to_string().blue(), p.display());
                            let count = self.passwords_file.import(p, t).context("密码导入失败")?;
                            self.passwords_file
                                .sort_and_write(&self.config.general.password_path)
                                .context("密码文件写入失败")?;
                            let msg = format!("已导入 {} 个密码", count.to_string().bold().green());
                            println!("{}", msg);
                            info!("{}", msg);
                        }
                    }
                } else {
                    // 若未指定密码动作时
                    if add.len() > 0 {
                        let count = self.passwords_file.add_passwords(add);
                        let msg = format!("增加 {} 个密码", count.to_string().bold().green());
                        println!("{}", msg);
                        info!("{}", msg);
                    };
                    if del.len() > 0 {
                        let count = self.passwords_file.del_passwords(del);
                        let msg = format!("删除 {} 个密码", count.to_string().bold().green());
                        println!("{}", msg);
                        info!("{}", msg);
                    };
                    self.passwords_file
                        .sort_and_write(&self.config.general.password_path)
                        .context("密码文件写入失败")?;
                    info!("已写入密码文件");
                }
            }
        }
        Ok(())
    }
}
