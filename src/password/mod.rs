//! # 密码模型
//! 提供密码的各种模型。

mod sample;

use std::io::Write;
use std::path::PathBuf;
use std::{fmt::Display, fs, path::Path};

use anyhow::{Context, Result};
use chrono::{Duration, Local, Utc};
use colored::Colorize;
use log::info;
use serde::{Deserialize, Serialize};

use crate::app::PasswordProcess;
use crate::config::ConvertConfig;
use crate::{config::PasswordConvertType, DEFAULT_REGEX};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PasswordManager<'a> {
    #[serde(skip_serializing, skip_deserializing)]
    pub runtime: Vec<&'a Password>,

    #[serde(skip_serializing, skip_deserializing)]
    pub recent: Vec<&'a Password>,

    #[serde(skip_serializing, skip_deserializing)]
    pub other: Vec<&'a Password>,

    #[serde(skip_serializing, skip_deserializing)]
    file_path: PathBuf,

    passwords: Vec<Password>,
}

impl<'a> Default for PasswordManager<'a> {
    fn default() -> Self {
        Self {
            runtime: Vec::new(),
            recent: Vec::new(),
            other: Vec::new(),
            file_path: PathBuf::new(),
            passwords: Vec::new(),
        }
    }
}

impl<'a> PasswordManager<'a> {
    /// 加载密码文件
    ///
    /// # Arguments
    ///
    /// - 'path' - 密码文件路径
    ///
    pub fn load(path: &Path) -> Result<Self> {
        let mut password_manager: Self = toml::from_str(&fs::read_to_string(path)?)?;
        password_manager.file_path = path.to_path_buf();
        password_manager
            .passwords
            .iter_mut()
            .for_each(|item| item.gen_escape());
        Ok(password_manager)
    }

    /// 按照使用次数降序排列
    fn sort_password(&mut self) {
        self.passwords.sort_by(|a, b| b.gmt_usage.cmp(&a.gmt_usage))
    }

    /// 根据密码热边界分类密码
    ///
    /// 根据传入的天数将多少天内使用的密码设置为常用密码提升优先级
    ///
    /// # Arguments
    ///
    /// - 'racent_days' - 热边界
    ///
    pub fn classify(&'a mut self, racent_days: i64) {
        let hot_boundary = Utc::now() - Duration::days(racent_days);
        self.recent = self
            .passwords
            .iter()
            .filter(|item| !item.priviliege && item.gmt_usage >= hot_boundary)
            .collect();
        // 按使用时间由近到远排列
        self.recent.sort_by(|a, b| b.gmt_usage.cmp(&a.gmt_usage));

        self.other = self
            .passwords
            .iter()
            .filter(|item| !item.priviliege && item.gmt_usage < hot_boundary)
            .collect();
        // 按使用时间由近到远排列
        self.other.sort_by(|a, b| b.gmt_usage.cmp(&a.gmt_usage));

        self.runtime = self
            .passwords
            .iter()
            .filter(|item| !item.priviliege)
            .collect();
        // 按使用时间由近到远排列
        self.runtime.sort_by(|a, b| b.gmt_usage.cmp(&a.gmt_usage));

        // self.passwords.iter().filter(|item| {
        //     if item.gmt_usage > hot_boundary {
        //         // 使用时间大于热边界的储存于频繁级别
        //         self.recent.push(item);
        //     } else {
        //         // 其他的储存于低优先级
        //         self.other.push(item);
        //     }
        // });
    }

    /// 显示所有密码
    pub fn display(&self) -> String {
        format!(
            "{}\n共 {} 个密码",
            self.passwords
                .iter()
                .map(|password| password.display())
                .collect::<Vec<String>>()
                .join("\n"),
            self.passwords.len().to_string().green(),
        )
    }

    /// 列出所有密码字符串
    pub fn list(&self) -> String {
        self.passwords
            .iter()
            .map(|item| item.as_str())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// 密码个数
    pub fn count(&self) -> usize {
        self.passwords.len()
    }

    /// 密码操作入口
    ///
    /// # Arguments
    ///
    /// - 'command' - 命令
    /// - 'add' - 增加的密码字符串列表
    /// - 'del' - 删除的密码字符串列表
    /// - 'config' - 配置
    ///
    pub fn run(
        &mut self,
        command: &Option<PasswordProcess>,
        add: &[String],
        del: &[String],
        config: &ConvertConfig,
    ) -> Result<()> {
        if let Some(command) = command {
            match command {
                PasswordProcess::List => {
                    // 列出密码
                    println!("{}", self.display());
                }
                PasswordProcess::Export {
                    export_type: _,
                    export_path: _,
                } => {
                    // 导出密码
                    let password_type = &config.export_type;
                    let path = &config.export_path;
                    if !path.exists() {
                        fs::File::create(&path)
                            .context(format!("导出文件创建失败: '{}'", &path.display()))?;
                    }

                    info!(
                        "导出密码[{}]: {}",
                        password_type.to_string().blue(),
                        path.display()
                    );
                    let count = self.export(path, password_type).context("密码导出失败")?;
                    let msg = format!("已导出 {} 个密码", count.to_string().bold().green());
                    println!("{}", msg);
                    info!("{}", msg);
                }
                PasswordProcess::Import {
                    import_type: _,
                    import_path: _,
                } => {
                    // 导入密码
                    let tpassword_type = &config.import_type;
                    let path = &config.import_path;
                    info!(
                        "导入密码[{}]: {}",
                        tpassword_type.to_string().blue(),
                        path.display()
                    );
                    let count = self.import(path, tpassword_type).context("密码导入失败")?;
                    self.write().context("密码文件写入失败")?;
                    let msg = format!("已导入 {} 个密码", count.to_string().bold().green());
                    println!("{}", msg);
                    info!("{}", msg);
                }
            }
        } else {
            // 若未指定密码动作时
            if add.len() > 0 {
                let count = self.add_passwords(add);
                let msg = format!("增加 {} 个密码", count.to_string().bold().green());
                println!("{}", msg);
                info!("{}", msg);
            };
            if del.len() > 0 {
                let count = self.del_passwords(del);
                let msg = format!("删除 {} 个密码", count.to_string().bold().green());
                println!("{}", msg);
                info!("{}", msg);
            };
            self.write().context("密码文件写入失败")?;
            info!("已写入密码文件");
        }
        Ok(())
    }

    /// 将密码写入磁盘
    ///
    /// Arguments
    ///
    /// - `path` - 密码文件路径
    ///
    pub fn write(&mut self) -> Result<()> {
        self.sort_password();
        fs::write(
            &self.file_path,
            sample::PASSWORDS.to_string() + &toml::to_string(self)?,
        )?;
        Ok(())
    }

    /// 创建示例密码文件
    ///
    /// Arguments
    ///
    /// - `path` - 密码文件路径
    ///
    pub fn write_sample(path: &Path) -> Result<()> {
        fs::File::create(path)?.write_all(sample::PASSWORDS.as_bytes())?;
        Ok(())
    }

    /// 向 runtime 级别增加引用
    ///
    /// 如果 password 列表中没有，则先增加。如果 runtime、recent、other 中已有引用，则删除并向 runtime 首部添加引用
    ///
    /// Arguments
    ///
    /// - `list` - 密码密码字符串列表
    ///
    pub fn add_privilege(&mut self, list: &[String]) -> usize {
        let mut add_list = Vec::new();
        list.into_iter().for_each(|item| {
            match self
                .passwords
                .iter_mut()
                .find(|password| &password.password == item)
            {
                Some(password) => {
                    password.priviliege = true;
                }
                None => {
                    add_list.push(item.to_string());
                }
            };
        });
        let count = self.add_passwords(&add_list);
        let now = Utc::now();
        add_list.into_iter().for_each(|item| {
            self.passwords.push(Password {
                password: item,
                escape: String::new(),
                priviliege: true,
                usage_count: 0,
                gmt_crate: now,
                gmt_usage: now,
            })
        });
        count
    }

    /// 将密码提升至 runtime 级别
    ///
    /// 如果 password 列表中没有，则会复制到列表中
    ///
    /// Arguments
    ///
    /// - `password` - 密码引用
    ///
    pub fn move_to_runtime(&'a mut self, password: &'a Password) {
        self.other.retain(|item| item != &password);
        self.recent.retain(|item| item != &password);
        self.runtime.retain(|item| item != &password);
        self.runtime.insert(0, password);
    }

    /// 添加密码
    ///
    /// Arguments
    ///
    /// - `list` - 添加的密码字符串列表
    ///
    /// Returns
    ///
    /// 成功添加的个数，不包括已存在的
    ///
    pub fn add_passwords(&mut self, list: &[String]) -> usize {
        if list.len() == 0 {
            return 0;
        };
        let prev = self.passwords.len();

        list.into_iter()
            .filter(|item| {
                self.passwords
                    .iter()
                    .all(|password| &password.password != *item)
            }) // 筛选未包含的密码
            .collect::<Vec<&String>>()
            .into_iter()
            .for_each(|item| self.passwords.push(Password::create(&item, 0)));
        self.passwords.len() - prev
    }

    /// 删除密码
    ///
    /// Arguments
    ///
    /// - `list` - 删除的密码字符串列表
    ///
    /// Returns
    ///
    /// 成功删除的个数，不包括不存在的
    ///
    pub fn del_passwords(&mut self, list: &[String]) -> usize {
        // 将索引列出
        let mut indexes = self
            .passwords
            .iter()
            .enumerate()
            .filter(|(_, password)| list.contains(&password.password)) // 筛选密码文件中命中的
            .fold(vec![], |mut acc, (index, _)| {
                acc.push(index);
                acc
            });

        // 由索引从大到小删除密码文件中的密码
        indexes.sort();
        indexes.reverse();
        indexes.iter().for_each(|i| {
            self.passwords.remove(*i);
        });

        indexes.len()
    }

    /// 导出密码
    ///
    /// Arguments
    ///
    /// - `path` - 导出路径
    /// - `convert_type` - 导出文件格式
    ///
    pub fn export(&self, path: &Path, convert_type: &PasswordConvertType) -> Result<usize> {
        let write_str: String;
        match convert_type {
            PasswordConvertType::Text => {
                write_str = self
                    .passwords
                    .iter()
                    .map(|password| password.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");
            }
            PasswordConvertType::Jtmdy => {
                write_str = self
                    .passwords
                    .iter()
                    .map(|password| format!("{}\t\t{}", password.to_string(), password.usage_count))
                    .collect::<Vec<String>>()
                    .join("\n");
            }
        }
        fs::write(path, write_str)?;
        Ok(self.passwords.len())
    }

    /// 导入密码
    ///
    /// Arguments
    ///
    /// - `path` - 导入路径
    /// - `convert_type` - 导入文件格式
    ///
    pub fn import(&mut self, path: &Path, convert_type: &PasswordConvertType) -> Result<usize> {
        let file_str = fs::read_to_string(path)?;

        match convert_type {
            PasswordConvertType::Text => {
                let verified = file_str
                    .split("\n")
                    .filter(|item| {
                        if item.len() == 0 {
                            false
                        } else {
                            self.passwords
                                .iter()
                                .all(|password| &password.password != item)
                        }
                    }) // 筛选不在密码文件中且有内容的密码
                    .collect::<Vec<&str>>();
                verified
                    .iter()
                    .for_each(|s| self.passwords.push(Password::create(s, 0)));
                Ok(verified.len())
            }
            PasswordConvertType::Jtmdy => {
                let prev = self.passwords.len();
                DEFAULT_REGEX
                    .pw_type_jtmdy
                    .captures_iter(&file_str)
                    .filter(|item| {
                        self.passwords
                            .iter()
                            .all(|password| &password.password != &item[1])
                    }) // 筛选不在密码文件中的密码
                    .collect::<Vec<_>>()
                    .into_iter()
                    .for_each(|item| {
                        self.passwords
                            .push(Password::create(&item[1], item[2].parse().unwrap_or(0)))
                    });
                Ok(self.passwords.len() - prev)
            }
        }
    }
}

/// # 密码结构
/// 提供密码的各种模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password {
    /// 密码字符串
    pub password: String,

    /// 命令行密码
    #[serde(skip_serializing, skip_deserializing)]
    pub escape: String,

    /// 提升优先级
    #[serde(skip_serializing, skip_deserializing)]
    priviliege: bool,

    /// 使用次数
    pub usage_count: usize,

    /// 创建时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub gmt_crate: chrono::DateTime<Utc>,

    /// 修改时间、上次使用时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub gmt_usage: chrono::DateTime<Utc>,
}

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.password)
    }
}

impl Password {
    /// 生成调用命令使用的转义密码
    fn gen_escape(&mut self) {
        self.escape = self.password.replace("\"", "\\\"")
    }

    /// 取得密码字符串引用
    pub fn as_str(&self) -> &str {
        &self.password
    }

    /// 显示密码
    pub fn display(&self) -> String {
        format!(
            "'{}' [使用次数: {}, 上次使用: {}]",
            self.to_string().bold().green(),
            self.usage_count.to_string().bold().blue(),
            self.gmt_usage.with_timezone(&Local).to_rfc3339().cyan()
        )
    }

    /// 按当前时间创建密码
    ///
    /// Arguments
    ///
    /// - `password` - 密码字符串
    /// - `count` - 使用次数
    ///
    pub fn create(password: &str, count: usize) -> Self {
        let now = Utc::now();
        let mut password = Self {
            password: password.to_string(),
            escape: String::new(),
            priviliege: false,
            usage_count: count,
            gmt_crate: now,
            gmt_usage: now,
        };
        password.gen_escape();
        password
    }

    /// 记录被使用
    ///
    /// 使用次数+1,使用时间设置为，当前时间
    ///
    pub fn used(&mut self) {
        self.usage_count += 1;
        self.gmt_usage = Utc::now();
    }
}
