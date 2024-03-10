//! # 密钥管理模块
//!
//! 提供密钥管理器，用于操作密钥的读取、存储、管理、排序。

use log::info;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use chrono::{Duration, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use super::{sample, Token};
use crate::TokenListStyle;

/// # 密钥管理器
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenManager {
    /// 本次使用过的密钥
    #[serde(skip_serializing, skip_deserializing)]
    pub runtime: Vec<Token>,

    /// 最近使用过的密钥
    #[serde(skip_serializing, skip_deserializing)]
    pub recent: Vec<Token>,

    /// 其他的密钥
    #[serde(skip_serializing, skip_deserializing)]
    pub other: Vec<Token>,

    /// 密钥文件路径
    #[serde(skip_serializing, skip_deserializing)]
    path: PathBuf,

    /// 所有的密钥
    tokens: Vec<Token>,
}

impl TokenManager {
    /// 显示所有密钥
    ///
    /// # Arguments
    ///
    /// - `list_style` - 密钥显示样式
    pub fn display(&self, list_style: &TokenListStyle) -> String {
        match list_style {
            TokenListStyle::Plain => self.list(),
            TokenListStyle::Detail => format!(
                "{}\n共 {} 个密钥",
                self.tokens
                    .iter()
                    .map(|item| item.display())
                    .collect::<Vec<String>>()
                    .join("\n"),
                self.count().to_string().green(),
            ),
        }
    }

    /// 列出所有密钥字符串
    pub fn list(&self) -> String {
        self.tokens
            .iter()
            .map(|item| item.as_str())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// 统计密钥数量
    pub fn count(&self) -> usize {
        self.tokens.len()
    }

    /// 加载密钥文件
    ///
    /// # Arguments
    ///
    /// - 'token_path' - 密钥文件路径
    pub fn load(&mut self, token_path: &Path) -> Result<()> {
        self.path = token_path.to_path_buf();
        // 如果文件不存在则创建示例
        if !token_path.exists() {
            File::create(token_path)?.write_all(sample::TOKENS.as_bytes())?;
            return Ok(());
        }
        let toml_token: Self = toml::from_str(&fs::read_to_string(token_path)?)?;
        self.tokens = toml_token.tokens;
        self.deduplicate();
        self.tokens.iter_mut().for_each(Token::gen_str);

        Ok(())
    }

    /// 密钥去重
    ///
    /// 将 tokens 字段中的密钥进行对比，两个密钥字符串一样时保留一个并更新以下字段：
    ///
    /// - 使用最远的创建时间
    /// - 使用最近的使用时间
    /// - 使用次数相加
    fn deduplicate(&mut self) {
        let mut hash_map: HashMap<String, Token> = HashMap::new();
        self.tokens.iter().for_each(|item| {
            if let Some(value) = hash_map.get_mut(&item.token) {
                info!("已合并密钥: {} -> {}", item.display(), value.display());
                value.usage_count += item.usage_count;
                value.gmt_crate = value.gmt_crate.min(item.gmt_crate);
                value.gmt_usage = value.gmt_usage.max(item.gmt_usage);
            } else {
                hash_map.insert(item.token.to_string(), item.clone());
            }
        });
        self.tokens = hash_map.into_values().collect();
    }

    /// 删除密钥
    ///
    /// # Arguments
    ///
    /// - `token_strs` - 需要删除的密钥字符串
    pub fn delete_tokens(&mut self, token_strs: &[String]) -> usize {
        if token_strs.len() == 0 {
            return 0;
        };
        let prev = self.count();
        self.tokens.retain(|item| !token_strs.contains(&item.token));
        prev - self.count()
    }

    /// 添加密钥
    ///
    /// # Arguments
    ///
    /// - `token_strs` - 需要添加的密钥字符串
    pub fn add_tokens(&mut self, token_strs: &[String]) -> usize {
        if token_strs.len() == 0 {
            return 0;
        };
        let mut add_tokens = token_strs
            .into_iter()
            .filter(|item| {
                self.tokens
                    .iter()
                    .all(|token_exists| &token_exists.token != *item)
            })
            .map(|item| Token::new(item))
            .collect::<Vec<Token>>();
        let add_count = add_tokens.len();
        self.tokens.append(&mut add_tokens);
        add_count
    }

    /// 根据热边界分类密钥
    ///
    /// 根据传入的天数将多少天内使用的密钥设置为常用
    ///
    /// # Arguments
    ///
    /// - 'racent_days' - 热边界
    pub fn classify(&mut self, racent_days: usize) {
        let hot_boundary = Utc::now() - Duration::days(racent_days as i64);
        self.recent = self
            .tokens
            .iter()
            .filter(|item| item.is_priviliege() && item.gmt_usage >= hot_boundary)
            .map(|item| item.clone())
            .collect();
        // 按使用时间由近到远排列
        self.recent.sort_by(|a, b| b.gmt_usage.cmp(&a.gmt_usage));

        self.other = self
            .tokens
            .iter()
            .filter(|item| item.is_priviliege() && item.gmt_usage < hot_boundary)
            .map(|item| item.clone())
            .collect();
        // 按使用次数由多至少排列
        self.other.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));

        self.runtime = self
            .tokens
            .iter()
            .filter(|item| item.is_priviliege())
            .map(|item| item.clone())
            .collect();
        // 按当前使用次数由多至少排列
        self.runtime
            .sort_by(|a, b| b.current_count.cmp(&a.current_count));
    }

    /// 提交密钥并写入磁盘
    pub fn commit_and_write(&mut self) -> Result<()> {
        self.commit();
        self.write()
    }

    /// 提交密钥
    pub fn commit(&mut self) {
        self.tokens = self
            .runtime
            .iter()
            .chain(&self.recent)
            .chain(&self.other)
            .map(|item| item.clone())
            .collect();
        (self.runtime, self.recent, self.other) = (vec![], vec![], vec![]);
        self.sort();
    }

    /// 将密钥写入磁盘
    pub fn write(&mut self) -> Result<()> {
        self.sort();
        fs::write(
            &self.path,
            sample::TOKENS.to_string() + &toml::to_string(self)?,
        )?;
        Ok(())
    }

    /// 按照使用次数降序排列
    fn sort(&mut self) {
        self.tokens
            .sort_by(|a, b| b.usage_count.cmp(&a.usage_count))
    }
}
