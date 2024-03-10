//! # 密钥管理模块
//!
//! 提供密钥管理器，用于操作密钥的读取、存储、管理、排序。

use std::fmt::Display;

use chrono::{Local, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};

mod manager;
mod sample;

pub use manager::TokenManager;

/// # 密钥
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// 密钥字符串
    pub token: String,

    /// 特权密钥
    #[serde(skip_serializing, skip_deserializing)]
    priviliege: bool,

    /// 命令行密钥字符串
    #[serde(skip_serializing, skip_deserializing)]
    pub token_str: String,

    /// 总计使用次数
    pub usage_count: usize,

    /// 本次使用次数
    #[serde(skip_serializing, skip_deserializing)]
    pub current_count: usize,

    /// 创建时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub gmt_crate: chrono::DateTime<Utc>,

    /// 修改时间、上次使用时间
    #[serde(with = "chrono::serde::ts_seconds")]
    pub gmt_usage: chrono::DateTime<Utc>,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.token)
    }
}

impl Token {
    /// 取得密钥引用
    pub fn as_str(&self) -> &str {
        &self.token
    }

    /// 显示密钥信息
    pub fn display(&self) -> String {
        // [${添加时间} | ${最近使用时间} | ${使用次数}] ${密钥}
        format!(
            "[{} | {} | {}] {}",
            self.gmt_crate.with_timezone(&Local).to_rfc3339().cyan(),
            self.gmt_usage.with_timezone(&Local).to_rfc3339().cyan(),
            self.usage_count.to_string().bold().blue(),
            self.to_string().bold().green(),
        )
    }

    /// 新建密钥
    ///
    /// Arguments
    ///
    /// - `token` - 密钥字符串
    pub fn new(token: &str) -> Self {
        let now = Utc::now();
        Self {
            token: token.to_string(),
            priviliege: false,
            token_str: String::new(),
            current_count: 0,
            usage_count: 0,
            gmt_crate: now,
            gmt_usage: now,
        }
    }

    /// 创建密钥
    ///
    /// Arguments
    ///
    /// - `token` - 密钥字符串
    /// - `priviliege` - 是否特权密钥
    /// - `count` - 总使用次数
    pub fn create(token: &str, priviliege: bool, count: usize) -> Self {
        let mut new_token = Token::new(token);
        new_token.priviliege = priviliege;
        new_token.usage_count = count;
        new_token
    }

    /// 记录密钥被使用一次
    pub fn use_once(&mut self) {
        self.current_count += 1;
        self.usage_count += 1;
        self.gmt_usage = Utc::now();
    }

    /// 生成转义后的密钥
    pub fn gen_str(&mut self) {
        self.token_str = self.token.replace("\"", "\\\"")
    }

    /// 判断是否特权密钥
    pub fn is_priviliege(&self) -> bool {
        self.priviliege
    }
}
