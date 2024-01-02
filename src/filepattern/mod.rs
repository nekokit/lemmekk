use std::cmp;

use anyhow::{anyhow, Result};

/// 压缩文件特征
pub const COMPRESSED_FEATURE: [(CompressType, &[u8]); 9] = [
    (CompressType::Zip, &[0x50, 0x4B, 0x03, 0x04]),
    (
        CompressType::Rar,
        &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00],
    ),
    (CompressType::SevenZ, &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
    (CompressType::Tar, &[0x75, 0x73, 0x74, 0x61, 0x72]),
    (CompressType::Xz, &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]),
    (CompressType::TarGz, &[0x1F, 0x8B, 0x08, 0x00]),
    (CompressType::TarBz2, &[0x42, 0x5A, 0x68, 0x39, 0x17]),
    (CompressType::Gzip, &[0x1F, 0x8B]),
    (CompressType::BZip2, &[0x42, 0x5A, 0x68]),
];
/// 图片文件特征
pub const IMAGE_FEATURE: [(ImageType, &[u8], &[u8]); 2] = [
    (ImageType::Jpeg, &[0xFF, 0xD8, 0xFF], &[0xFF, 0xD9]),
    (
        ImageType::Png,
        &[0x89, 0x50, 0x4E, 0x47],
        &[0xAE, 0x42, 0x60, 0x82],
    ),
];

/// 压缩文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressType {
    Zip,
    Rar,
    SevenZ,
    Tar,
    Xz,
    TarGz,
    TarBz2,
    Gzip,
    BZip2,
}

/// 图片类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
    Jpeg,
    Png,
}

pub fn is_match_compressed_header(source: &[u8]) -> bool {
    COMPRESSED_FEATURE.iter().any(|item| {
        if item.1.len() > source.len() {
            return false;
        } else {
            &source[..item.1.len()] == item.1
        }
    })
}

/// 匹配压缩文件头
pub fn match_compressed_header(source: &[u8]) -> Result<CompressType> {
    let mut ret = Err(anyhow!(
        "不支持的压缩格式: {:?}",
        &source[..cmp::min(8, source.len())]
    ));
    COMPRESSED_FEATURE.iter().for_each(|item| {
        if source.len() >= item.1.len() && &source[..item.1.len()] == item.1 {
            ret = Ok(item.0)
        }
    });
    ret
}

/// 匹配图片文件头
pub fn match_image_header(source: &[u8]) -> Result<ImageType> {
    let mut ret = Err(anyhow!(
        "不支持的图片隐写格式: {:?}",
        &source[..cmp::min(8, source.len())]
    ));
    IMAGE_FEATURE.iter().for_each(|item| {
        if source.len() >= item.1.len() && &source[..item.1.len()] == item.1 {
            ret = Ok(item.0)
        }
    });
    ret
}

pub fn get_image_feature(
    image_type: ImageType,
) -> Result<(ImageType, &'static [u8], &'static [u8])> {
    let mut ret = Err(anyhow!("未注册的图片格式"));
    IMAGE_FEATURE.iter().for_each(|(tp, h, t)| {
        if image_type == *tp {
            ret = Ok((*tp, *h, *t));
        };
    });
    ret
}
