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

# 解压配置
[extract]

# 解压配置：配置 7zip 程序路径
#
# 配置 7zip 程序路径，若为空则直接调用 `7z`，请确认 7zip 执行文件所在文件夹已经加入环境变量 `PATH`。
path_7z = ''

# 解压输入配置：需解压的文件或目录
#
# 路径使用半角单引号包裹可不用转义，使用半角双引号包裹需要转义。
source = ['test-data']

# 解压配置：文件夹搜索深度
#
# 输入路径为文件夹时的搜索深度，范围：(-128,127]。
#
# - 负数 - 搜索所有子文件夹
# - 0 - 只搜索输入文件夹下的直系文件，默认
# - 正数 n - 搜索至输入文件夹的第 n 代文件夹
search_depth = 0

# 解压配置：需排除的文件扩展名
#
# 符合的文件将不会解压
# 扩展名半角单引号包裹，半角逗号+空格或换行分隔，例: `['{扩展名}', '{扩展名}']`
excluded_suffix = [
    # office 文件
    'doc', 'docx', 'docm', 'docz', 'dot', 'dotx', 'dotm',
    'xls', 'xlsx', 'xlsm', 'xlsz', 'xlt', 'xltx', 'xltm',
    'ppt', 'pptx', 'pptm', 'pptz', 'pot', 'potx', 'potm',
    'wps', 'msg', 'odt', 'ods', 'odp',
    # java
    'jar', 'jarx', 'war', 'xpi',
    # 安装包
    'msi', 'cab', 'cabinet', 'deb', 'rpm', 'ipk',
    'crx', 'apk', 'bar', 'xap', 'ipa', 'pkg',
    # 资源包
    'pk3', 'pk4', 'vpk', 'pak', 'zap',
    'sav', 'save',
    # 镜像
    'iso', 'udf', 'mdf', 'mds', 'wim', 'img', 'bin',
    # 多媒体
    'epub', 'apng', 'amz',
    # 其余自定义
    ]

# 解压配置：优先使用的密钥
#
# 密钥使用半角单引号包裹，半角逗号+空格或换行分隔，例: `['密钥1', '密钥2']`
# 优先级：运行时
# 指定的密钥将会添加到密钥文件
tokens = []

# 解压配置：常用密钥存留时间
#
# 一定时间内使用过密钥的优先级将提升至常用级，默认 `30`，单位：天
token_hot_boundary = 30

# 解压配置：解压目标文件夹
#
# 路径使用半角单引号包裹可不用转义，使用半角双引号包裹需要转义。
otutput_dir = ''

# 解压配置：解压后对压缩文件的操作
#
# 可选：
#
# - `DoNothing` - 不做任何事，默认
# - `Delete` - 删除
# - `Move` - 移动
defer_operation = "DoNothing"

# 解压配置：回收目录
#
# 解压后选择移动压缩文件时，将压缩文件移动到的目录。
recycle_dir = ''

# 解压选项
[extract.method]

# 解压选项：识别图种隐写文件
#
# 是否识别图种隐写文件，默认为 false
recogniz_steganography = false

# 解压选项：直接解压
#
# 是否直接解压到目标文件夹，不为每个压缩包创建目录，默认为 false
extract_directly = false

# 解压选项：智能直接解压
#
# 在 解压选项：直接解压 关闭（永远创建文件夹）时，该选项失效
# 是否智能直接解压，在压缩包内只有单文件/单文件夹的情况下，不创建文件夹直接解压，默认为 false
extract_directly_single = false

# 解压选项：递归解压
#
# 是否递归解压压缩文件内的压缩文件，默认为 false
recursively = false

"#;
