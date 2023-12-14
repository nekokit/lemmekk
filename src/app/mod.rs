//! # 应用程序

use std::{error::Error, fs, path::PathBuf};

use colored::Colorize;
use log::{debug, info};

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
    pub fn create(args: CliArgs) -> Result<Self, Box<dyn Error>> {
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

    pub fn run(&'a mut self) -> Result<(), Box<dyn Error>> {
        debug!("{:?}", self.args);
        debug!("{:?}", self.config);

        self.config.check_general()?;
        self.passwords_file = PasswordFile::load(&self.config.general.password_path)?;
        info!(
            "已读取密码 {} 个",
            self.passwords_file.count().to_string().bold().yellow()
        );

        match &self.args.command {
            args::Command::Extract {
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
                todo!()
            }
            args::Command::Password { command, add, del } => {
                if let Some(command) = command {
                    match command {
                        args::PasswordProcess::List => {
                            println!("{}", self.passwords_file.display())
                        }
                        args::PasswordProcess::Export {
                            export_type,
                            export_path,
                        } => {
                            if let Some(t) = export_type {
                                self.config.convert.export_type = t.clone();
                            }
                            if let Some(p) = export_path {
                                self.config.convert.export_path = p.clone();
                            }
                            let t = &self.config.convert.export_type;
                            let p = &self.config.convert.export_path;
                            if !p.exists() {
                                fs::File::create(&p)?;
                            }

                            info!("导出密码[{}]: {}", t.to_string().blue(), p.display());
                            let count = self.passwords_file.export(p, t)?;
                            self.passwords_file
                                .write(&self.config.general.password_path)?;
                            info!("已导出 {} 个密码", count.to_string().bold().yellow())
                        }
                        args::PasswordProcess::Import {
                            import_type,
                            import_path,
                        } => {
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
                            info!("已导入 {} 个密码", count.to_string().bold().yellow())
                        }
                    }
                } else {
                    if add.len() > 0 {
                        let count = self.passwords_file.add_passwords(add);
                        info!("增加 {} 个密码", count.to_string().bold().yellow());
                    };
                    if del.len() > 0 {
                        let count = self.passwords_file.del_passwords(del);
                        info!("删除 {} 个密码", count.to_string().bold().yellow());
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
