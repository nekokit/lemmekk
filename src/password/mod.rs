//! # 密码模型
//! 提供密码的各种模型。

mod sample;

use std::io::Write;
use std::{fmt::Display, fs, path::Path};

use anyhow::Result;
use chrono::{Local, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{config::PasswordConvertType, DEFAULT_REGEX};

/// # 密码结构
/// 提供密码的各种模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password {
    /// 密码字符串
    pub password: String,

    /// 命令行密码
    #[serde(skip_serializing, skip_deserializing)]
    pub pw_for_7z: String,

    /// 使用次数
    pub usage_count: usize,

    /// 创建时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub gmt_crate: chrono::DateTime<Utc>,

    /// 修改时间、上次使用时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub gmt_usage: chrono::DateTime<Utc>,
}

impl Password {
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

    pub fn create(password: &str, count: usize) -> Self {
        let pw_for_7z = password.replace("\"", "\\\"");
        let now = Utc::now();
        Self {
            password: password.to_string(),
            pw_for_7z,
            usage_count: count,
            gmt_crate: now,
            gmt_usage: now,
        }
    }

    pub fn used(&mut self) {
        self.usage_count += 1;
        self.gmt_usage = Utc::now();
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.password)
    }
}

/// # 经过分类的密码引用
#[derive(Debug)]
pub struct PasswordList<'a> {
    /// 运行时使用的密码
    pub runtime: Vec<&'a mut Password>,
    /// 近期使用过的密码
    pub frequent: Vec<&'a mut Password>,
    /// 剩余的密码
    pub others: Vec<&'a mut Password>,
}

impl<'a> PasswordList<'a> {
    pub fn new() -> Self {
        Self {
            runtime: vec![],
            frequent: vec![],
            others: vec![],
        }
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
    /// 加载密码文件
    ///
    /// # Arguments
    ///
    /// - `path` - 密码文件路径
    ///
    pub fn load(path: &Path) -> Result<PasswordFile> {
        if !path.exists() {}
        let mut passwords: PasswordFile = toml::from_str(&fs::read_to_string(path)?)?;
        passwords.gen_pw_for_7z();
        Ok(passwords)
    }

    /// 生成命令行密码
    fn gen_pw_for_7z(&mut self) {
        self.passwords.iter_mut().for_each(|item| {
            item.pw_for_7z = item.password.replace("\"", "\\\"");
        });
    }

    /// 取得现有密码个数
    pub fn count(&self) -> usize {
        self.passwords.len()
    }

    /// 将密码按照最后使用时间降序排列并写入磁盘
    ///
    /// Arguments
    ///
    /// - `path` - 密码文件路径
    ///
    pub fn sort_and_write(&mut self, path: &Path) -> Result<()> {
        // 按照最后使用时间降序排列
        self.passwords.sort_by(|a, b| b.gmt_usage.cmp(&a.gmt_usage));
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
    pub fn write_sample(path: &Path) -> Result<()> {
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
            .map(|item| Password::create(&item, 0)) // 转为 Password 对象
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
    pub fn export(&self, path: &Path, convert_type: &PasswordConvertType) -> Result<usize> {
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
    pub fn import(&mut self, path: &Path, convert_type: &PasswordConvertType) -> Result<usize> {
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
                    .map(|s| Password::create(s, 0))
                    .for_each(|p| self.passwords.push(p));
                Ok(verified.len())
            }
            PasswordConvertType::Jtmdy => {
                let file_str = fs::read_to_string(path)?;
                let verified = DEFAULT_REGEX
                    .pw_type_jtmdy
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
