//! # CLI 参数与行为模块

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{TokenFilePattern, TokenListStyle};

/// CLI 参数
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "让我康康\n这是一个基于 7zip 的批量解压的工具，可使用密钥文件匹配压缩包密码。\n支持的压缩格式有 rar4, rar5, zip, 7z, xz 等，同时支持图种检测、递归解压并提供密钥管理等功能。"
)]
pub struct CliArgs {
    /// 配置选项：使用指定的配置文件
    ///
    /// 默认配置文件根据操作系统不同位于不同位置：
    ///
    /// - windows - `${程序文件夹}\config.toml`
    /// - linux/macos - `~/.config/lemmekk/comfig.toml`
    #[arg(
        short,
        long,
        help = "使用指定的配置文件",
        long_help = "使用指定的配置文件，默认配置文件根据操作系统不同位于不同位置：\nwindows: `${程序文件夹}\\config.toml`\nlinux/macos: `~/.config/lemmekk/comfig.toml`。"
    )]
    pub config: Option<PathBuf>,

    /// 配置选项：使用指定的密钥文件
    ///
    /// 默认密钥文件根据操作系统不同位于不同位置：
    ///
    /// - windows - `${程序文件夹}\default.token`
    /// - linux/macos - `~/.config/lemmekk/default.token`
    #[arg(
        short,
        long,
        help = "使用指定的密钥文件",
        long_help = "使用指定的密钥文件，默认密钥文件文件根据操作系统不同位于不同位置：\nwindows: `${程序文件夹}\\default.token`\nlinux/macos: `~/.config/lemmekk/default.token`。"
    )]
    pub token: Option<PathBuf>,

    /// 主要命令
    #[command(subcommand)]
    pub main_command: MainCommand,
}

/// 主要命令模块
#[derive(Debug, Subcommand)]
pub enum MainCommand {
    /// # 密钥处理模块
    #[command(about = "密钥处理模块")]
    Token {
        /// # 密钥导入导出
        #[command(subcommand)]
        command: Option<TokenProcess>,

        /// 密钥选项：添加密钥
        ///
        /// 可多次指定添加多个密钥，密钥存在时不做处理。
        #[arg(
            short,
            long,
            help = "添加密钥",
            long_help = "添加密钥，可多次指定添加多个密钥，密钥存在时不做处理"
        )]
        add: Vec<String>,

        /// 密钥选项：删除密钥
        ///
        /// 可多次指定删除多个密钥，密钥不存在时不做处理。
        #[arg(
            short,
            long,
            help = "删除密钥",
            long_help = "删除密钥，可多次指定删除多个密钥，密钥不存在时不做处理"
        )]
        delete: Vec<String>,
    },
}

/// # 密钥导入导出模块
#[derive(Debug, Subcommand)]
pub enum TokenProcess {
    /// # 列出密钥
    #[command(about = "列出密钥")]
    List {
        /// 列出选项：指定列出的密钥模式
        ///
        /// 可选：
        ///
        /// - **`plain`** - 文本模式，默认
        /// - `detail` - 详细信息模式
        #[arg(short, long, help = "指定列出的密钥模式")]
        style: Option<TokenListStyle>,
    },

    /// # 导出密钥
    #[command(about = "导出密钥")]
    Export {
        /// 导出选项：指定导出的密钥模式
        ///
        /// 可选：
        ///
        /// - **`plain`** - 文本模式，默认
        /// - `jtmdy` - 解TMD压模式
        #[arg(short, long, help = "指定导出的密钥模式")]
        pattern: Option<TokenFilePattern>,

        /// 导出选项：指定导出的密钥路径
        ///
        /// 默认：`~/.config/lemmekk/token.txt`
        #[arg(short, long, help = "指定导出的密钥路径")]
        file: Option<PathBuf>,
    },

    /// # 导入密钥
    ///
    /// 命令中的选项均为可选，未指定时均使用配置中的值。
    #[command(about = "导入密钥")]
    Import {
        /// 导入选项：指定导入的密钥格式
        ///
        /// 可选：
        ///
        /// - **plain** - 文本模式，默认
        /// - jtmdy - 解TMD压模式
        #[arg(short, long, help = "指定导入的密钥模式")]
        pattern: Option<TokenFilePattern>,

        /// 导入选项：指定导入的密钥路径
        ///
        /// 默认：`~/.config/lemmekk/token.txt`
        #[arg(short, long, help = "指定导入的密钥路径")]
        file: Option<PathBuf>,
    },
}
