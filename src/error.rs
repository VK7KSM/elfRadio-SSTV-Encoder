//! 错误类型定义
//! 
//! 本模块定义了SSTV库中使用的所有错误类型，提供了统一的错误处理机制。

use thiserror::Error;

/// SSTV编码器错误类型
#[derive(Error, Debug)]
pub enum SstvError {
    /// 图像处理错误
    #[error("图像处理失败: {0}")]
    ImageError(#[from] image::ImageError),

    /// 音频处理错误
    #[error("音频处理失败: {0}")]
    AudioError(#[from] hound::Error),

    /// IO错误
    #[error("文件操作失败: {0}")]
    IoError(#[from] std::io::Error),

    /// 不支持的SSTV模式
    #[error("不支持的SSTV模式: {mode}")]
    UnsupportedMode { mode: String },

    /// 无效的采样率
    #[error("无效的采样率: {sample_rate}Hz，支持范围: {min_rate}-{max_rate}Hz")]
    InvalidSampleRate {
        sample_rate: u32,
        min_rate: u32,
        max_rate: u32,
    },

    /// 调制过程错误
    #[error("SSTV调制失败: {message}")]
    ModulationError { message: String },

    /// 内存分配错误
    #[error("内存不足: 需要 {required} 字节")]
    MemoryError { required: usize },

    /// 图像处理错误（通用）
    #[error("图像处理失败: {0}")]
    ImageProcessing(String),

    /// 无效的音频参数
    #[error("无效的音频参数: {parameter} = {value}")]
    InvalidAudioParameter { parameter: String, value: String },

    /// 无效的音频格式
    #[error("无效的音频格式: {0}")]
    InvalidFormat(String),
}

/// 库的Result类型别名
pub type Result<T> = std::result::Result<T, SstvError>;

impl SstvError {
    /// 创建不支持的模式错误
    pub fn unsupported_mode<S: Into<String>>(mode: S) -> Self {
        Self::UnsupportedMode { mode: mode.into() }
    }

    /// 创建无效采样率错误
    pub fn invalid_sample_rate(sample_rate: u32, min_rate: u32, max_rate: u32) -> Self {
        Self::InvalidSampleRate {
            sample_rate,
            min_rate,
            max_rate,
        }
    }

    /// 创建调制错误
    pub fn modulation_error<S: Into<String>>(message: S) -> Self {
        Self::ModulationError {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = SstvError::unsupported_mode("TestMode");
        assert!(matches!(err, SstvError::UnsupportedMode { .. }));
    }

    #[test]
    fn test_error_display() {
        let err = SstvError::invalid_sample_rate(22050, 8000, 48000);
        let error_string = format!("{}", err);
        assert!(error_string.contains("22050"));
        assert!(error_string.contains("8000"));
        assert!(error_string.contains("48000"));
    }
}