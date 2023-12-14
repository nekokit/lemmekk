//！ # 让我康康
//！ 这是一个基于 7zip 的批量解压的工具，可使用密码字典匹配压缩包密码。
//！ 支持的压缩格式有 rar4, rar5, zip, 7z, xz 等，同时支持图种检测、递归解压、密码管理等功能。

use std::{
    env,
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

mod app;
mod config;
mod log;
mod password;

pub use app::Application;
pub use app::CliArgs;
pub use password::Password;
pub use password::PasswordFile;
pub use password::PasswordList;

/// 默认路径提供器
pub const DEFAULT_PATH: PathLoader = PathLoader;

/// # 路径提供器
/// 调用方法返回相应的路径。
pub struct PathLoader;
impl PathLoader {
    /// # 程序路径
    pub fn program(&self) -> PathBuf {
        let mut path = env::current_exe().expect("读取程序路径错误");
        path.pop();
        path
    }
    /// # 工作路径
    pub fn pwd(&self) -> PathBuf {
        env::current_dir().expect("读取工作路径错误")
    }
    /// # 配置文件路径
    pub fn config(&self) -> PathBuf {
        self.program().join("config.toml")
    }
    /// # 密码文件路径
    pub fn password(&self) -> PathBuf {
        self.program().join("passwords.toml")
    }
    /// # 日志文件路径
    pub fn log(&self) -> PathBuf {
        self.program().join("result.log")
    }
    /// # 密码导入导出文件路径
    pub fn exchange(&self) -> PathBuf {
        self.program().join("passwords.txt")
    }
}

/// 按行读取
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
