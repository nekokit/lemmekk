//! # 应用程序
use std::{fs, path::PathBuf};

use colored::Colorize;
use log::{debug, info};

use crate::AppError;
use crate::{config::Config, PasswordFile, PasswordList, DEFAULT_PATH};

mod args;
pub use args::CliArgs;
pub use args::Command;
pub use args::PasswordProcess;

/// # 应用程序对象
pub struct Application<'a> {
    /// 来自 cli 的参数
    args: CliArgs,
    /// 执行的配置
    pub config: Config,
    /// 密码列表
    pub passwords: PasswordList<'a>,
    /// 密码文件
    passwords_file: PasswordFile,
    /// 待解压列表
    pending_files: Vec<PathBuf>,
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
    pub fn create(args: CliArgs) -> Result<Self, AppError> {
        // 优先从程序参数获取配置文件路径，否则使用默认
        let path = match &args.config_file {
            Some(p) => p.clone(),
            None => DEFAULT_PATH.config(),
        };
        Config::check_path(&path)?;
        let config = Config::load(&path)?.overlay(&args);

        Ok(Self {
            args,
            config,
            passwords: PasswordList::new(),
            pending_files: vec![],
            passwords_file: PasswordFile::new(),
        })
    }

    /// 主程序运行逻辑
    pub fn run(&'a mut self) -> Result<(), AppError> {
        debug!("{:?}", self.args);
        debug!("{:?}", self.config);

        self.config.general.check()?;
        self.passwords_file = PasswordFile::load(&self.config.general.password_path)?;
        info!(
            "已读取密码 {} 个",
            self.passwords_file.count().to_string().bold().yellow()
        );

        match &self.args.command {
            Command::Extract {
                path_for_7z: _,
                extract_input: _,
                extract_output: _,
                passwords: _,
                operation_for_extracted: _,
                dir_for_move: _,
                extract_directly: _,
                extract_directly_single: _,
                recursively: _,
            } => {
                self.config.extract.check()?;
            }
            Command::Password { command, add, del } => {
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
                                fs::File::create(&path)?;
                            }

                            info!(
                                "导出密码[{}]: {}",
                                password_type.to_string().blue(),
                                path.display()
                            );
                            let count = self.passwords_file.export(path, password_type)?;
                            self.passwords_file
                                .write(&self.config.general.password_path)?;
                            let msg =
                                format!("已导出 {} 个密码", count.to_string().bold().yellow());
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
                            let count = self.passwords_file.import(p, t)?;
                            self.passwords_file
                                .write(&self.config.general.password_path)?;
                            let msg =
                                format!("已导入 {} 个密码", count.to_string().bold().yellow());
                            println!("{}", msg);
                            info!("{}", msg);
                        }
                    }
                } else {
                    // 若未指定密码动作时
                    if add.len() > 0 {
                        let count = self.passwords_file.add_passwords(add);
                        let msg = format!("增加 {} 个密码", count.to_string().bold().yellow());
                        println!("{}", msg);
                        info!("{}", msg);
                    };
                    if del.len() > 0 {
                        let count = self.passwords_file.del_passwords(del);
                        let msg = format!("删除 {} 个密码", count.to_string().bold().yellow());
                        println!("{}", msg);
                        info!("{}", msg);
                    };
                    self.passwords_file
                        .write(&self.config.general.password_path)?;
                    info!("已写入密码文件");
                }
            }
        }
        Ok(())
    }
}
