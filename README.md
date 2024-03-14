# Lemmekk 让我康康

## 简介

lemmekk 目标是一个跨平台的工具。主要功能为调用 `7z` 批量解压文件，同时集成简单的密钥管理功能。

lemmekk 还在开发中，目标特性如下：

- [x] 简单密钥管理
  - [x] 列出密钥
  - [x] 添加密钥
  - [x] 删除密钥
  - [x] 导入导出
    - [x] 普通文本
    - [x] `解TMD压` 格式
- [x] 解压模块
  - [x] 排除指定扩展名
  - [x] 识别分卷
  - [x] 识别图种
  - [x] 识别压缩包
  - [ ] 递归解压
  - [ ] 解压后操作
    - [ ] 移动
    - [ ] 删除

## 依赖

- 7zip

解压需要调用 7zip，可通过以下两种方式任选其一进行关联。

### 通过环境变量

#### windows

在系统中安装好 7zip 后，将 7zip 程序路径（包含以下文件的文件夹）加入环境变量 `PATH`，
若在命令提示符/终端中运行 `7z`，显示 7zip 的帮助信息即完成。

#### linux/macos

建议通过包管理安装 7zip，在终端中执行 `7z` 出现 7zip 的帮助提示即可。
若未出现则需要将 7zip 执行文件所在文件夹加入 `PATH`。


### 通过配置文件指定 7zip 路径

将以下文件的路径添加到工具配置文件：

- 7z / 7z.exe
- 7za / 7za.exe


## 安装

通过页面右侧 releases 下载编译好的执行文件，
解压后放入 `PATH` 中的路径，
或将所在文件夹也加入 `PATH`，
又或在所在文件夹中打开终端。

建议使用前两者，今后可在任意位置运行命令。

在终端中运行：

```shell
lemmekk help
```

会在终端中显示帮助信息。

## 配置

第一次可运行列出密钥命令，工具在默认位置找不到所需配置时会生成示例文件。

```shell
lemmekk token list
```

生成以下文件：

windows：

| 文件名        | 文件类型 | 位置                      |
| ------------- | -------- | ------------------------- |
| config.toml   | 配置文件 | ${工具执行文件所在文件夹} |
| default.token | 密钥文件 | ${工具执行文件所在文件夹} |
| result.log    | 日志     | ${工具执行文件所在文件夹} |

linus/macos

| 文件名        | 文件类型 | 位置               |
| ------------- | -------- | ------------------ |
| config.toml   | 配置文件 | ~/.config/lemmekk/ |
| default.token | 密钥文件 | ~/.config/lemmekk/ |
| result.log    | 日志     | ~/.config/lemmekk/ |

详细配置可参考示例配置文件中的注释。

各处配置的优先级：命令行参数 > 配置文件 > 默认值

可以在配置文件中指定大部分选项，命令行只指定操作命令就行了，如：

```shell
lemmekk token list
```

## 密钥管理

```
lemmekk token [-选项 参数] [操作命令]
```

### 选项

- `-a` / `--add`: 添加密钥
  - 在 `操作命令` 为空时生效，可多次指定一次添加多个密钥，值包含特殊字符时使用引号包裹。
  - 以 `-` 开头的密钥可能会因为解析无法添加，可通过直接编辑 `default.token` 进行添加。
  - 例：`lemmekk token -a 密码1 -a " 密 码2" -a '密码3"'`
- `-d` / `--delete`: 移除密钥
  - 在 `操作命令` 为空时生效，可多次指定一次移除多个密钥，值包含特殊字符时使用引号包裹。
  - 以 `-` 开头的密钥可能会因为解析无法添加，可通过直接编辑 `default.token` 进行移除。
  - 例：`lemmekk token -d 密码1 -d " 密 码2" -d '密码3"'`

### 操作命令

#### `list` - 列出密钥

选项：

- `-s` / `--style`: 列出的密钥样式，可在配置文件中指定
  - 为满足今后拓展或与脚本、其他工具协作，使用了选项以提供不同的输出格式。
  - **值 `plain`**: 只列出密钥字符串，默认
  - 值 `detail`: 列出详细信息，包括添加使用时间、使用次数等信息
  - 例：`lemmekk token list`、`lemmekk token list -s detail`

例子：

```shell
lemmekk token export
lemmekk token export -p jtmdy -f pw.txt
```

#### `export` - 导出密钥

选项：

- `-p` / `--pattern`: 导出的密钥样式，可在配置文件中指定
  - 为满足今后拓展或与脚本、其他工具协作，使用了选项以提供不同的输出格式。
  - **值 `plain`**: 只列出密钥字符串，默认
  - 值 `jtmdy`: 导出为 `解TMD压` 密码本模式：`${密钥}\t\t${使用次数}`
- `-f` / `--file`: 导出的密钥文件位置，可在配置文件中指定

例子：

```shell
lemmekk token export
lemmekk token export -p jtmdy -f pw.txt
```

#### `import` - 导入密钥

选项：

- `-p` / `--pattern`: 导入的密钥样式，可在配置文件中指定
  - 为满足今后拓展或与脚本、其他工具协作，使用了选项以提供不同的输出格式。
  - **值 `plain`**: 一行内容是为一个密钥字符串，默认
  - 值 `jtmdy`: 导入 `解TMD压` 密码本
- `-f` / `--file`: 导入的密钥文件位置，可在配置文件中指定

在导入文件时，如果有匹配到其他模式的情况，会阻止导入，如确定需要增加，请直接编辑密钥文件。

例子：

```shell
lemmekk token import
lemmekk token import -p jtmdy -f pw.txt
```

### 密钥文件

密钥文件 `default.token` 本质上是一个 Toml 文件，可使用任意文本编辑器进行编辑，
语法详见 [Toml 官网](https://toml.io/cn/)。

```toml
# 示例密钥文件
# 密钥格式:
[[tokens]]
# 密钥字符串
# 若使用半角单引号包裹，密钥内不能再使用半角单引号
# 若使用半角双引号包裹，密钥内不能再使用半角双引号，并且特殊字符需要转义
# 详见 TOML 字符串: https://toml.io/cn/v1.0.0#%E5%AD%97%E7%AC%A6%E4%B8%B2
token = 'sample'
# 使用次数
usage_count = 0
# 添加时间，值为 UNIX 时间戳
gmt_crate = 1701360000
# 最后使用时间，值为 UNIX 时间戳
gmt_modified = 1701360000
```

其中 `gmt_crate`、`gmt_modified` 只影响在解压过程中的排序，可以复制其他密钥的时间戳，
或使用 [Unix Time Stamp](https://www.unixtimestamp.com/zh/) 生成指定时间的时间戳