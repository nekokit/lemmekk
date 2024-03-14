//! 信息模块
//!
//! 提供工具的信息及展示方式

use std::fmt::Display;

use anyhow::bail;
use colored::Colorize;

/// 工具信息载体
pub struct AppInfo {
    name: String,
    module_name: String,
    module_version: Version,
    lib_version: Version,
}

impl Display for AppInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{} {} (lib {})",
            self.name, self.module_name, self.module_version, self.lib_version
        )
    }
}
impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: String::from("让我康康"),
            module_name: Default::default(),
            module_version: Default::default(),
            lib_version: Version {
                major: 0,
                minor: 1,
                revision: 6,
                edition: Edition::Dev,
            },
        }
    }
}
impl AppInfo {
    /// 输出工具信息
    pub fn display(&self) -> String {
        let module_version_colored = match self.module_version.edition {
            Edition::Dev => self.module_version.to_string().red(),
            Edition::Beta => self.module_version.to_string().yellow(),
            Edition::Stable => self.module_version.to_string().green(),
        };
        let lib_version_colored = match self.lib_version.edition {
            Edition::Dev => self.lib_version.to_string().red(),
            Edition::Beta => self.lib_version.to_string().yellow(),
            Edition::Stable => self.lib_version.to_string().green(),
        };
        format!(
            "{}-{} {} (lib {})",
            self.name, self.module_name, module_version_colored, lib_version_colored
        )
    }

    /// 设置模块名称
    ///
    /// # Arguments
    ///
    /// - `name` - 模块名称
    pub fn set_module_name(&mut self, name: &str) {
        self.module_name = name.to_string();
    }

    /// 设置模块版本号
    ///
    /// # Arguments
    ///
    /// - `major` - 主版本号
    /// - `minor` - 次版本号
    /// - `revision` - 修订号
    /// - `edition` - 版本类型（dev、beta、stable）
    pub fn set_module_version(&mut self, major: u8, minor: u8, revision: u8, edition: &str) {
        self.module_version = Version {
            major,
            minor,
            revision,
            edition: edition.try_into().unwrap_or(Edition::Dev),
        }
    }
}

struct Version {
    major: u8,
    minor: u8,
    revision: u8,
    edition: Edition,
}
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "v{}.{}.{} {}",
            self.major, self.minor, self.revision, self.edition
        )
    }
}
impl Default for Version {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 0,
            revision: 0,
            edition: Edition::Dev,
        }
    }
}

enum Edition {
    Dev,
    Beta,
    Stable,
}
impl Display for Edition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Edition::Dev => write!(f, "Dev"),
            Edition::Beta => write!(f, "Beta"),
            Edition::Stable => write!(f, "Stable"),
        }
    }
}
impl TryFrom<&str> for Edition {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "DEV" => Ok(Self::Dev),
            "BETA" => Ok(Self::Beta),
            "STABLE" => Ok(Self::Stable),
            _ => bail!("版本号 Edition 转换错误"),
        }
    }
}
