//! # CLI 参数与行为模块

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{DeferOperation, TokenFilePattern, TokenListStyle};

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
            long_help = "添加密钥，可多次指定添加多个密钥，密钥存在时不做处理。"
        )]
        add: Vec<String>,

        /// 密钥选项：删除密钥
        ///
        /// 可多次指定删除多个密钥，密钥不存在时不做处理。
        #[arg(
            short,
            long,
            help = "删除密钥",
            long_help = "删除密钥，可多次指定删除多个密钥，密钥不存在时不做处理。"
        )]
        delete: Vec<String>,
    },
    /// # 解压模块
    #[command(about = "解压处理模块")]
    Extract {
        /// 解压输入配置：需解压的文件或目录
        ///
        /// 路径使用半角单引号包裹可不用转义，使用半角双引号包裹需要转义。
        #[arg(
            short = 'i',
            long,
            help = "需解压的文件或目录",
            long_help = "需解压的文件或目录，可多次指定。"
        )]
        source: Vec<PathBuf>,

        /// 解压配置：文件夹搜索深度
        ///
        /// 输入路径为文件夹时的搜索深度。
        ///
        /// - 负数 - 搜索所有子文件夹
        /// - 0 - 只搜索输入文件夹下的直系文件，默认
        /// - 正数 n - 搜索至输入文件夹的第 n 代文件夹
        #[arg(
            short,
            long,
            help = "文件夹搜索深度",
            long_help = "文件夹搜索深度，输入路径为文件夹时的搜索深度，范围：(-128,127]。\n负数 - 为搜索所有子文件夹。\n`0` - 只搜索输入文件夹下的直系文件，默认。\n正数 n - 搜索至输入文件夹的第 n 代文件夹。"
        )]
        search_depth: Option<i8>,

        /// 解压配置：需排除的文件扩展名
        ///
        /// 符合的文件将不会解压
        /// 扩展名半角单引号包裹，半角逗号+空格或换行分隔，例: `['{扩展名}', '{扩展名}']`
        #[arg(
            short,
            long,
            help = "需排除的文件扩展名",
            long_help = "需排除的文件扩展名，符合的文件将不会解压。"
        )]
        excluded_suffix: Vec<String>,

        /// 解压配置：优先使用的密钥
        ///
        /// 密钥使用半角单引号包裹，半角逗号+空格或换行分隔，例: `['密钥1', '密钥2']`
        /// 优先级：运行时
        /// 指定的密钥将会添加到密钥文件
        #[arg(
            short,
            long,
            help = "优先使用的密钥",
            long_help = "优先使用的密钥，指定的密钥将会添加到密钥文件。"
        )]
        tokens: Vec<String>,

        /// 解压配置：常用密钥存留时间
        ///
        /// 一定时间内使用过密钥的优先级将提升至常用级，默认 `30`，单位：天
        #[arg(
            short = 'h',
            long,
            help = "常用密钥存留时间",
            long_help = "一定时间内使用过密钥的优先级将提升至常用级，默认 `30`，单位：天。"
        )]
        token_hot_boundary: Option<usize>,

        /// 解压配置：解压目标文件夹
        ///
        /// 路径使用半角单引号包裹可不用转义，使用半角双引号包裹需要转义。
        #[arg(short, long, help = "解压目标文件夹", long_help = "解压目标文件夹。")]
        otutput_dir: Option<PathBuf>,

        /// 解压配置：解压后对压缩文件的操作
        ///
        /// 可选：
        ///
        /// - `DoNothing` - 不做任何事，默认
        /// - `Delete` - 删除
        /// - `Move` - 移动
        #[arg(
            short,
            long,
            help = "解压后对压缩文件的操作",
            long_help = "解压后对压缩文件的操作，可选：\n- `DoNothing` - 不做任何事，默认\n- `Delete` - 删除\n- `Move` - 移动"
        )]
        defer_operation: Option<DeferOperation>,

        /// 解压配置：回收文件夹
        ///
        /// 解压后选择移动压缩文件时，将压缩文件移动到的目录。
        #[arg(
            short = 'm',
            long,
            help = "回收文件夹",
            long_help = "回收文件夹，解压后选择移动压缩文件时，将压缩文件移动到的目录。"
        )]
        recycle_dir: Option<PathBuf>,

        /// 解压选项：识别图片隐写文件
        ///
        /// 是否识别图片隐写文件，默认为 false
        #[arg(
            long,
            help = "识别图片隐写文件",
            long_help = "是否识别图片隐写文件，默认为 false。"
        )]
        analyze_steganography: Option<bool>,

        /// 解压选项：直接解压
        ///
        /// 是否直接解压到目标文件夹，不为每个压缩包创建目录，默认为 false
        #[arg(
            long,
            help = "直接解压",
            long_help = "是否直接解压到目标文件夹，不为每个压缩包创建目录，默认为 false。"
        )]
        extract_directly: Option<bool>,

        /// 解压选项：智能直接解压
        ///
        /// 在 解压选项：直接解压 关闭（永远创建文件夹）时，该选项失效
        /// 是否智能直接解压，在压缩包内只有单文件/单文件夹的情况下，不创建文件夹直接解压，默认为 false
        #[arg(
            long,
            help = "智能直接解压",
            long_help = "是否智能直接解压，在 直接解压 关闭（永远创建文件夹）时，该选项失效，在压缩包内只有单文件/单文件夹的情况下，不创建文件夹直接解压，默认为 false。"
        )]
        smart_directly: Option<bool>,

        /// 解压选项：递归解压
        ///
        /// 是否递归解压压缩文件内的压缩文件，默认为 false
        #[arg(
            long,
            help = "递归解压",
            long_help = "是否递归解压压缩文件内的压缩文件，默认为 false"
        )]
        recursively: Option<bool>,
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
