//! # 密码模型
//! 提供密码的各种模型。

use std::{fmt::Display, fs, path::Path};

use chrono::{Duration, Local, Utc};
use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{config::PasswordConvertType, AppError};

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

#[derive(Debug, Serialize, Deserialize)]
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

impl PasswordFile {
    pub fn new() -> Self {
        Self { passwords: vec![] }
    }
    pub fn load(path: &Path) -> Result<PasswordFile, AppError> {
        if !path.exists() {}
        let passwords: PasswordFile = toml::from_str(&fs::read_to_string(path)?)?;
        Ok(passwords)
    }
    pub fn count(&self) -> usize {
        self.passwords.len()
    }
    pub fn write(&self, path: &Path) -> Result<(), AppError> {
        fs::write(path, &toml::to_string(self)?)?;
        Ok(())
    }
    pub fn write_sample(&self, path: &Path) -> Result<(), AppError> {
        fs::write(path, &toml::to_string(self)?)?;
        Ok(())
    }
    pub fn add_passwords(&mut self, list: &[String]) -> usize {
        if list.len() < 1 {
            return 0;
        };
        let start = self.passwords.len();

        list.into_iter()
            .filter(|item| {
                !self
                    .passwords
                    .iter()
                    .any(|password| &password.password == *item)
            })
            .map(|item| Password::new(&item))
            .collect::<Vec<Password>>()
            .into_iter()
            .for_each(|item| self.passwords.push(item));

        self.passwords.len() - start
    }

    pub fn del_passwords(&mut self, list: &[String]) -> usize {
        let indexs = self
            .passwords
            .iter()
            .enumerate()
            .filter(|(_, password)| list.contains(&password.password))
            .fold(vec![], |mut acc, (index, _)| {
                acc.push(index);
                acc
            });
        indexs.iter().for_each(|i| {
            self.passwords.remove(*i);
        });

        indexs.len()
    }

    pub fn display(&self) -> String {
        self.passwords
            .iter()
            .map(|password| password.display())
            .collect::<Vec<String>>()
            .join("\n")
    }

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

    pub fn import(
        &mut self,
        path: &Path,
        convert_type: &PasswordConvertType,
    ) -> Result<usize, AppError> {
        match convert_type {
            PasswordConvertType::Text => {
                let file_str = fs::read_to_string(path)?;
                let current_passwords = self
                    .passwords
                    .iter()
                    .map(|p| p.as_str())
                    .collect::<Vec<&str>>();
                let verified = file_str
                    .split("\n")
                    .filter(|s| {
                        if s.len() == 0 {
                            false
                        } else {
                            !current_passwords.contains(s)
                        }
                    })
                    .collect::<Vec<&str>>();
                verified
                    .iter()
                    .map(|s| Password::new(s))
                    .for_each(|p| self.passwords.push(p));
                Ok(verified.len())
            }
            PasswordConvertType::Jtmdy => {
                let r = Regex::new(r"(.+)\t\t(\d+)")?;
                let file_str = fs::read_to_string(path)?;
                let current_passwords = self
                    .passwords
                    .iter()
                    .map(|p| p.as_str())
                    .collect::<Vec<&str>>();
                let verified = r
                    .captures_iter(&file_str)
                    .filter(|item| !current_passwords.contains(&&item[1]))
                    .map(|item| Password::create(&item[1], item[2].parse().unwrap_or(0)))
                    .collect::<Vec<Password>>();
                verified.iter().for_each(|p| self.passwords.push(p.clone()));
                Ok(verified.len())
            }
        }
    }
}
