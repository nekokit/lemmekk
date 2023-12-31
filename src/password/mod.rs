//! # 密码模型
//! 提供密码的各种模型。

mod sample;

use std::io::Write;
use std::{fmt::Display, fs, path::Path};

use chrono::{Duration, Local, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{config::PasswordConvertType, AppError, DEFAULT_REGEX};

/// # 密码结构
/// 提供密码的各种模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password {
    /// 密码字符串
    pub password: String,

    /// 使用次数
    pub usage_count: usize,

    /// 创建时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub gmt_crate: chrono::DateTime<Utc>,

    /// 修改时间、上次使用时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub gmt_modified: chrono::DateTime<Utc>,
}

impl Password {
    pub fn new(password: &str) -> Self {
        Self {
            password: password.to_string(),
            usage_count: 0,
            gmt_crate: Utc::now(),
            gmt_modified: Utc::now(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.password
    }

    /// 显示密码
    pub fn display(&self) -> String {
        format!(
            "'{}' [使用次数: {}, 上次使用: {}]",
            self.to_string().bold().yellow(),
            self.usage_count.to_string().bold().blue(),
            self.gmt_modified.with_timezone(&Local).to_rfc3339().green()
        )
    }

    pub fn create(password: &str, count: usize) -> Self {
        Self {
            password: password.to_string(),
            usage_count: count,
            gmt_crate: Utc::now(),
            gmt_modified: Utc::now(),
        }
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.password)
    }
}

/// # 经过分类的密码引用
pub struct PasswordList<'a> {
    /// 参数传递的密码
    pub args: Vec<&'a Password>,
    /// 运行时使用过的密码
    pub runtime: Vec<&'a Password>,
    /// 近期使用过的密码
    pub frequent: Vec<&'a Password>,
    /// 剩余的密码
    pub others: Vec<&'a Password>,
}

impl<'a> PasswordList<'a> {
    pub fn new() -> Self {
        Self {
            args: vec![],
            runtime: vec![],
            frequent: vec![],
            others: vec![],
        }
    }

    pub fn from_password_file(file: &'a PasswordFile) -> Self {
        let now = Utc::now();
        let mut list = Self::new();
        file.passwords
            .iter()
            .for_each(|item| match now - item.gmt_modified {
                v if v <= Duration::weeks(5) => list.frequent.push(item),
                _ => list.others.push(item),
            });
        list
    }
}

/// # 密码文件中存储的密码
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct PasswordFile {
    pub passwords: Vec<Password>,
}

impl From<&[Password]> for PasswordFile {
    fn from(value: &[Password]) -> Self {
        Self {
            passwords: value.to_vec(),
        }
    }
}

impl Default for PasswordFile {
    fn default() -> Self {
        Self { passwords: vec![] }
    }
}

impl PasswordFile {
    pub fn new() -> Self {
        Self { passwords: vec![] }
    }

    /// 加载密码文件
    ///
    /// # Arguments
    ///
    /// - `path` - 密码文件路径
    ///
    pub fn load(path: &Path) -> Result<PasswordFile, AppError> {
        if !path.exists() {}
        let passwords: PasswordFile = toml::from_str(&fs::read_to_string(path)?)?;
        Ok(passwords)
    }

    /// 取得现有密码个数
    pub fn count(&self) -> usize {
        self.passwords.len()
    }

    /// 取得密码字符串引用列表
    fn get_str_list(&self) -> Vec<&str> {
        self.passwords.iter().map(|p| p.as_str()).collect()
    }

    /// 将密码写入磁盘
    ///
    /// Arguments
    ///
    /// - `path` - 密码文件路径
    ///
    pub fn write(&self, path: &Path) -> Result<(), AppError> {
        fs::write(
            path,
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
    pub fn write_sample(path: &Path) -> Result<(), AppError> {
        fs::File::create(path)?.write_all(sample::PASSWORDS.as_bytes())?;
        Ok(())
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
                !self
                    .passwords
                    .iter()
                    .any(|password| &password.password == *item)
            }) // 筛选未包含的密码
            .map(|item| Password::new(&item)) // 转为 Password 对象
            .collect::<Vec<Password>>()
            .into_iter()
            .for_each(|item| self.passwords.push(item)); // 添加进密码文件

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
        let indexes = self
            .passwords
            .iter()
            .enumerate()
            .filter(|(_, password)| list.contains(&password.password)) // 筛选密码文件中命中的
            .fold(vec![], |mut acc, (index, _)| {
                acc.push(index);
                acc
            }); // 将索引列出
                // 删除密码文件中的密码
        indexes.iter().for_each(|i| {
            self.passwords.remove(*i);
        });

        indexes.len()
    }

    /// 显示密码
    pub fn display(&self) -> String {
        if self.count() == 0 {
            return "无密码".to_string();
        };
        self.passwords
            .iter()
            .map(|password| password.display())
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// 导出密码
    ///
    /// Arguments
    ///
    /// - `path` - 导出路径
    /// - `convert_type` - 导出文件格式
    ///
    pub fn export(
        &self,
        path: &Path,
        convert_type: &PasswordConvertType,
    ) -> Result<usize, AppError> {
        match convert_type {
            PasswordConvertType::Text => {
                let write_str = self
                    .passwords
                    .iter()
                    .map(|password| password.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");
                fs::write(path, write_str)?;
            }
            PasswordConvertType::Jtmdy => {
                let write_str = self
                    .passwords
                    .iter()
                    .map(|password| format!("{}\t\t{}", password.to_string(), password.usage_count))
                    .collect::<Vec<String>>()
                    .join("\n");
                fs::write(path, write_str)?;
            }
        }
        Ok(self.passwords.len())
    }

    /// 导入密码
    ///
    /// Arguments
    ///
    /// - `path` - 导入路径
    /// - `convert_type` - 导入文件格式
    ///
    pub fn import(
        &mut self,
        path: &Path,
        convert_type: &PasswordConvertType,
    ) -> Result<usize, AppError> {
        match convert_type {
            PasswordConvertType::Text => {
                let file_str = fs::read_to_string(path)?;
                let verified = file_str
                    .split("\n")
                    .filter(|item| {
                        if item.len() == 0 {
                            false
                        } else {
                            !self
                                .passwords
                                .iter()
                                .any(|password| &password.password == item)
                        }
                    }) // 筛选不在密码文件中且有内容的密码
                    .collect::<Vec<&str>>();
                verified
                    .iter()
                    .map(|s| Password::new(s))
                    .for_each(|p| self.passwords.push(p));
                Ok(verified.len())
            }
            PasswordConvertType::Jtmdy => {
                let r = DEFAULT_REGEX.pw_type_jtmdy();
                let file_str = fs::read_to_string(path)?;
                let verified = r
                    .captures_iter(&file_str)
                    .filter(|item| {
                        !self
                            .passwords
                            .iter()
                            .any(|password| &password.password == &item[1])
                    }) // 筛选不在密码文件中的密码
                    .map(|item| Password::create(&item[1], item[2].parse().unwrap_or(0)))
                    .collect::<Vec<Password>>();
                verified.iter().for_each(|p| self.passwords.push(p.clone()));
                Ok(verified.len())
            }
        }
    }
}
