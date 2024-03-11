//! # 密钥样板

pub const TOKENS: &str = r#"# 示例密钥文件
# 密钥格式:
#[[tokens]]
# 密钥字符串
# 若使用半角单引号包裹，密钥内不能再使用半角单引号
# 若使用半角双引号包裹，密钥内不能再使用半角双引号，并且特殊字符需要转义
# 详见 TOML 字符串: https://toml.io/cn/v1.0.0#%E5%AD%97%E7%AC%A6%E4%B8%B2
#token = 'sample'
# 使用次数
#usage_count = 0
# 添加时间，值为 UNIX 时间戳
#gmt_crate = 1701360000
# 最后使用时间，值为 UNIX 时间戳
#gmt_modified = 1701360000

"#;
