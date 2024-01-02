pub const CONFIG: &str = r#"# 示例配置文件
# 全局配置
[general]

# 日志等级，可选：
# - `Trace`
# - `Debug`
# - `Info` [默认]
# - `Warn`
# - `Error`
# log_level = "Trace"

# 日志存放路径
# 不设置时默认使用工具目录下的 `result.log`
# log_file = ''

# toml 密码字典路径
# 不设置时默认使用工具目录下的 `passwords.toml`
# password_file = ''

# 解压模块配置
[extract]

# 7z 程序路径
# 不设置时默认直接调用 `7z`，请确认 7z 执行文件位于环境变量 Path 中。
# path_for_7z = ''

# 需解压的文件或目录
# 路径使用半角单引号包裹，半角逗号+空格或换行分隔，例: `['{待解压文件}', '{待解压路径}']`
extract_input = []

# 是否搜索子文件夹中的文件，默认为 false
# walk_input = false

# 解压目标文件夹
extract_output = ''

# 使用指定的密码
# 密码使用半角单引号包裹，半角逗号+空格或换行分隔，例: `['{密码1}', '{密码2}']`
# 优先级：运行时
# 指定的密码将会添加到程序的密码字典
passwords = []

# 常用密码存留时间
# 一定时间内使用过密码的优先级将提升至常用级，单位：天
password_hot_boundary = 30

# 解压方式配置
[extract.extract_method]

# 解压后对压缩文件的操作，可选：
# - `DoNothing` - 不做任何事 [默认]
# - `Delete` - 删除
# - `Move` - 移动
operation_for_extracted = "DoNothing"

# 解压后选择移动时，将压缩文件移动到的目录。
dir_for_move = ''

# 是否不为每个压缩包创建目录，直接解压到目标文件夹，默认为 false
# extract_directly = false
# 在压缩包内只有单文件单文件夹的情况下，是否不创建文件夹直接解压，默认为 false
# extract_directly_single = false
# 是否递归解压压缩文件内的压缩文件，默认为 false
# recursively = false

# 密码模块配置
[convert]
# 密码导入配置
# 字典类型，可选：
# - Text: 普通密码字典，每行仅包含一个密码，例：`{密码}\n`
# - Jtmdy: 解TMD压格式，每行一个密码，密码后接两个 `tab` 后再接使用次数，例：`{密码}\t\t{使用次数}`
import_type = "Text"
# 导入字典路径，默认使用工具目录下的 `password.txt`
# import_path = ''

# 密码导出配置
# 字典类型，可选：
# - Text: 普通密码字典，每行仅包含一个密码，例：`{密码}\n`
# - Jtmdy: 解TMD压格式，每行一个密码，密码后接两个 `tab` 后再接使用次数，例：`{密码}\t\t{使用次数}`
export_type = "Text"
# 导出字典路径，默认使用工具目录下的 `password.txt`
# export_path = ''
"#;
