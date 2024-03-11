//! # 配置样板

pub const CONFIG: &str = r#"# 示例配置文件
# 全局配置
[general]

# 配置选项：使用指定的密钥文件
#
# 默认密钥文件根据操作系统不同位于不同位置：
#
# - windows - `${程序文件夹}\default.token`
# - linux/macos - `~/.config/lemmekk/default.token`
token = ''

# 密钥配置
[token]

# 列出选项：指定列出的密钥模式
#
# 可选：
#
# - **`Plain`** - 文本模式，默认
# - `Detail` - 详细信息模式
list_style = 'Plain'


# 导出选项：指定导出的密钥模式
#
# 可选：
#
# - **`Plain`** - 文本模式，默认
# - `Jtmdy` - 解TMD压模式
export_pattern = 'Plain'

# 导出选项：指定导出的密钥路径
#
# 默认：`~/.config/lemmekk/token.txt`
export_file = ''

# 导入选项：指定导入的密钥格式
#
# 可选：
#
# - **Plain** - 文本模式，默认
# - Jtmdy - 解TMD压模式
import_pattern = 'Plain'

# 导入选项：指定导入的密钥路径
#
# 默认：`~/.config/lemmekk/token.txt`
import_file = ''

"#;
