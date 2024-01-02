//! # CLI 参数解析
//! 运行参数行为解析。

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    config::{OperationForExtracted, PasswordConvertType},
    log::LogLevel,
};

/// # CLI 参数对象
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "让我康康\n这是一个基于 7zip 的批量解压的工具，可使用密码文件匹配压缩包密码。\n支持的压缩格式有 rar4, rar5, zip, 7z, xz 等，同时支持图种检测、递归解压、密码管理等功能。"
)]
pub struct CliArgs {
    /// 配置选项：使用指定的配置文件，默认配置文件位于 `程序文件夹` - `config.toml` 。
    #[arg(
        short,
        long,
        help = "使用指定的配置文件",
        long_help = "使用指定的配置文件，默认配置文件位于 `程序文件夹` - `config.toml`。"
    )]
    pub config_file: Option<PathBuf>,

    /// 配置选项：指定日志等级，可选:
    /// - `Trace` - 追踪等级
    /// - `Debug` - 调试等级
    /// - `Info` - 提示等级
    /// - `Warn` - 警告等级
    /// - `Error` - 错误等级
    #[arg(
        short,
        long,
        help = "使用指定的日志等级",
        long_help = "使用指定的日志等级，可选: `Trace`, `Debug`, `Warn`, `Error`。"
    )]
    pub log_level: Option<LogLevel>,

    /// 配置选项：使用指定的日志文件，默认配置文件位于 `程序文件夹` - `result.log` 。
    #[arg(
        long,
        help = "使用指定的日志文件",
        long_help = "使用指定的日志文件，默认配置文件位于 `程序文件夹` - `result.log`。"
    )]
    pub log_file: Option<PathBuf>,

    /// 配置选项：使用指定的密码文件，默认密码文件位于 `程序文件夹` - `password.toml` 。
    #[arg(
        short,
        long,
        help = "使用指定的密码文件",
        long_help = "使用指定的密码文件，默认密码文件位于 `程序文件夹` - `password.toml`。"
    )]
    pub password_file: Option<PathBuf>,

    /// # 模块
    #[command(subcommand)]
    pub command: Command,
}

/// # 模块
#[derive(Debug, Subcommand)]
pub enum Command {
    /// # 解压模块
    #[command(about = "解压模块")]
    Extract {
        /// 解压选项：指定 7z 程序路径，直接调用 `7z`，请确认 7z 执行文件位于环境变量 `Path` 中。
        #[arg(
            long,
            help = "指定 7z 程序路径",
            long_help = "指定 7z 程序路径，直接调用 `7z`，请确认 7z 执行文件位于环境变量 `Path` 中。"
        )]
        path_for_7z: Option<PathBuf>,

        /// 解压选项：指定需解压的文件或目录，可多次指定输入多个文件或文件夹。
        #[arg(
            short = 'i',
            long,
            help = "指定需解压的文件或目录",
            long_help = "指定需解压的文件或目录，可多次指定输入多个文件或文件夹。"
        )]
        extract_input: Vec<PathBuf>,

        /// 解压选项：[开关] 是否搜索子文件夹中的文件。
        #[arg(short, long, help = "[开关] 是否搜索子文件夹中的文件")]
        walk_input: Option<bool>,

        /// 解压选项：需排除的文件扩展名。
        #[arg(short, long, help = "需排除的文件扩展名")]
        excluded_extension: Vec<String>,

        /// 解压选项：指定输出文件夹，文件夹不存在时将会被创建。
        #[arg(
            short = 'o',
            long,
            help = "指定解压目标文件夹",
            long_help = "指定解压目标文件夹，文件夹不存在时将会被创建。"
        )]
        extract_output: Option<PathBuf>,

        /// 解压选项：指定使用的密码，可多次指定，这些密码将会存储到密码文件中。
        #[arg(
            short = 'p',
            long,
            help = "指定使用的密码",
            help = "指定使用的密码，可多次指定，这些密码将会存储到密码文件中。"
        )]
        passwords: Vec<String>,

        /// 解压选项：指定解压完成后对压缩文件的操作。
        /// 可选:
        /// - `do-nothing` - 不做任何处理
        /// - `delete` - 删除
        /// - `move` - 移动
        #[arg(
            short = 'j',
            long,
            value_enum,
            help = "指定解压完成后操作",
            long_help = "指定解压完成后操作。"
        )]
        operation_for_extracted: Option<OperationForExtracted>,

        /// 解压选项：当解压完成后的操作指定为 `move` 时，将压缩文件移动到的目录。
        #[arg(
            short = 'd',
            long,
            help = "指定将压缩文件移动到的目录",
            long_help = "当解压完成后的操作指定为 `move` 时，将压缩文件移动到的目录。"
        )]
        dir_for_move: Option<PathBuf>,

        /// 解压选项：[开关] 是否不为每个压缩包创建目录，直接解压到目标文件夹。
        #[arg(long, help = "[开关] 是否识别图种隐写文件")]
        recogniz_steganography: Option<bool>,

        /// 解压选项：[开关] 是否不为每个压缩包创建目录，直接解压到目标文件夹。
        #[arg(long, help = "[开关] 是否不为每个压缩包创建目录，直接解压到目标文件夹")]
        extract_directly: Option<bool>,

        /// 解压选项：[开关] 在压缩包内只有单文件单文件夹的情况下，是否不创建文件夹直接解压。
        #[arg(
            long,
            help = "[开关] 在压缩包内只有单文件单文件夹的情况下，是否不创建文件夹直接解压"
        )]
        extract_directly_single: Option<bool>,

        /// 解压选项：[开关] 是否递归解压压缩文件内的压缩文件。
        #[arg(short = 'r', long, help = "[开关] 是否递归解压压缩文件内的压缩文件")]
        recursively: Option<bool>,
    },

    /// # 密码处理模块
    #[command(about = "密码处理模块")]
    Password {
        /// # 密码导入导出
        #[command(subcommand)]
        command: Option<PasswordProcess>,

        /// 密码选项：添加密码
        /// 可多次指定添加多个密码，密码存在时不做处理。
        #[arg(
            short,
            long,
            help = "添加密码",
            long_help = "添加密码，可多次指定添加多个密码，密码存在时不做处理"
        )]
        add: Vec<String>,

        /// 密码选项：删除密码
        /// 可多次指定删除多个密码，密码不存在时不做处理。
        #[arg(
            short,
            long,
            help = "删除密码",
            long_help = "删除密码，可多次指定删除多个密码，密码不存在时不做处理"
        )]
        del: Vec<String>,
    },
}

/// # 密码导入导出模块
#[derive(Debug, Subcommand)]
pub enum PasswordProcess {
    /// # 列出密码
    #[command(about = "列出")]
    List,

    /// # 导出密码
    #[command(about = "导出密码")]
    Export {
        /// 导出选项：指定导出的密码格式
        #[arg(short = 't', long, help = "指定导出的密码样式")]
        export_type: Option<PasswordConvertType>,

        /// 导出选项：指定导出的密码路径
        #[arg(short = 'p', long, help = "指定导出的密码路径")]
        export_path: Option<PathBuf>,
    },

    /// # 导入密码
    /// 命令中的选项均为可选，未指定时均使用配置中的值。
    #[command(about = "导入密码")]
    Import {
        /// 导入选项：指定导入的密码格式
        #[arg(short = 't', long, help = "指定导入的密码样式")]
        import_type: Option<PasswordConvertType>,

        /// 导入选项：指定导入的密码路径
        #[arg(short = 'p', long, help = "指定导入的密码路径")]
        import_path: Option<PathBuf>,
    },
}
