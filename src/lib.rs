//! # SSTV-Rust
//!
//! 高性能SSTV（慢扫描电视）调制器Rust库
//!
//! 本库提供了完整的SSTV图像到音频信号的调制功能，基于C语言参考实现，
//! 支持标准SSTV模式，具有相位连续性和采样精度补偿等高级优化。
//!
//! ## 特性
//!
//! - 支持标准SSTV模式（Scottie-DX, Robot-36, PD-120, Martin-M1）
//! - 相位连续性优化算法
//! - 采样精度补偿技术
//! - 高性能音频生成和处理
//! - 完整的错误处理和类型安全
//! - WAV文件导出功能
//!
//! ## 使用示例
//!
//! ```rust
//! use sstv_rust::{SstvModulator, SstvMode, WavWriter};
//! use image::open;
//!
//! // 加载图像
//! let image = open("example.jpg").unwrap();
//!
//! // 创建调制器
//! let mut modulator = SstvModulator::new(SstvMode::Robot36);
//!
//! // 调制图像为Robot-36模式
//! modulator.modulate_image(&image).unwrap();
//!
//! // 导出WAV文件
//! modulator.export_wav("output.wav").unwrap();
//! ```
//!
//! ## 便捷函数
//!
//! ```rust
//! use sstv_rust::{generate_sstv_from_file, SstvMode};
//!
//! // 一行代码生成SSTV音频
//! generate_sstv_from_file("input.jpg", "output.wav", SstvMode::ScottieDx).unwrap();
//! ```

pub mod sstv;
pub mod audio;
pub mod error;

// 重新导出主要类型
pub use error::{SstvError, Result};
pub use sstv::{SstvMode, SstvModulator, ImageSaveConfig, ProcessingMetadata, MemoryUsage, MemoryUsageMB};
pub use audio::{AudioGenerator, WavWriter, effects};

/// 库版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 默认采样率 - 优化为6kHz以减少文件大小（基于奈奎斯特定理，SSTV最大频率2.5kHz）
pub const DEFAULT_SAMPLE_RATE: u32 = 6000;

/// 便捷函数：从图像文件直接生成SSTV音频
///
/// # 参数
/// * `image_path` - 输入图像文件路径
/// * `output_path` - 输出WAV文件路径
/// * `mode` - SSTV模式
///
/// # 示例
/// ```rust
/// use sstv_rust::{generate_sstv_from_file, SstvMode};
///
/// generate_sstv_from_file("input.jpg", "output.wav", SstvMode::Robot36).unwrap();
/// ```
pub fn generate_sstv_from_file<P1, P2>(
    image_path: P1,
    output_path: P2,
    mode: SstvMode,
) -> Result<()>
where
    P1: AsRef<std::path::Path>,
    P2: AsRef<std::path::Path>,
{
    // 加载图像
    let image = image::open(image_path)
        .map_err(|e| SstvError::ImageProcessing(format!("无法加载图像: {}", e)))?;

    // 创建调制器并调制
    let mut modulator = SstvModulator::new(mode);
    modulator.modulate_image(&image)?;

    // 导出WAV文件
    modulator.export_wav(output_path)?;

    Ok(())
}

/// 便捷函数：从内存中的图像数据生成SSTV音频
///
/// # 参数
/// * `image` - 图像数据
/// * `output_path` - 输出WAV文件路径
/// * `mode` - SSTV模式
///
/// # 示例
/// ```rust
/// use sstv_rust::{generate_sstv_from_image, SstvMode};
/// use image::DynamicImage;
///
/// let image = image::open("test.png").unwrap();
/// generate_sstv_from_image(&image, "output.wav", SstvMode::ScottieDx).unwrap();
/// ```
pub fn generate_sstv_from_image<P>(
    image: &image::DynamicImage,
    output_path: P,
    mode: SstvMode,
) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    // 创建调制器并调制
    let mut modulator = SstvModulator::new(mode);
    modulator.modulate_image(image)?;

    // 导出WAV文件
    modulator.export_wav(output_path)?;

    Ok(())
}

/// 获取SSTV模式信息
///
/// # 返回
/// 返回所有支持的SSTV模式及其详细信息：(模式, 名称, 尺寸, 传输时间)
pub fn get_supported_modes() -> Vec<(SstvMode, &'static str, (u32, u32), f64)> {
    vec![
        (SstvMode::ScottieDx, "Scottie-DX", SstvMode::ScottieDx.get_dimensions(), SstvMode::ScottieDx.get_duration()),
        (SstvMode::Robot36, "Robot-36", SstvMode::Robot36.get_dimensions(), SstvMode::Robot36.get_duration()),
        (SstvMode::Pd120, "PD-120", SstvMode::Pd120.get_dimensions(), SstvMode::Pd120.get_duration()),
        (SstvMode::MartinM1, "Martin-M1", SstvMode::MartinM1.get_dimensions(), SstvMode::MartinM1.get_duration()),
    ]
}

/// 计算SSTV传输的估计文件大小
///
/// # 参数
/// * `mode` - SSTV模式
/// * `sample_rate` - 采样率 (Hz)
/// * `bit_depth` - 位深度
///
/// # 返回
/// 估计的WAV文件大小（字节）
pub fn estimate_file_size(mode: SstvMode, sample_rate: u32, bit_depth: u16) -> usize {
    let duration = mode.get_duration();
    let sample_count = (duration * sample_rate as f64) as usize;
    let bytes_per_sample = (bit_depth / 8) as usize;
    sample_count * bytes_per_sample + 44 // WAV头部大小
}

/// 便捷函数：生成SSTV音频并保存处理后的图片
///
/// # 参数
/// * `image_path` - 输入图像文件路径
/// * `output_dir` - 输出目录路径
/// * `base_name` - 基础文件名
/// * `mode` - SSTV模式
/// * `image_config` - 图片保存配置
///
/// # 返回
/// 返回 (音频文件路径, 图片文件路径)
///
/// # 示例
/// ```rust
/// use sstv_rust::{generate_sstv_with_image_save, SstvMode, ImageSaveConfig};
///
/// let (audio_path, image_path) = generate_sstv_with_image_save(
///     "input.jpg", 
///     "output", 
///     "test", 
///     SstvMode::Robot36,
///     &ImageSaveConfig::png()
/// ).unwrap();
/// ```
pub fn generate_sstv_with_image_save<P1, P2, P3>(
    image_path: P1,
    output_dir: P2,
    base_name: P3,
    mode: SstvMode,
    image_config: &ImageSaveConfig,
) -> Result<(std::path::PathBuf, std::path::PathBuf)>
where
    P1: AsRef<std::path::Path>,
    P2: AsRef<std::path::Path>,
    P3: AsRef<str>,
{
    // 加载图像
    let image = image::open(image_path)
        .map_err(|e| SstvError::ImageProcessing(format!("无法加载图像: {}", e)))?;

    // 创建调制器并批处理
    let mut modulator = SstvModulator::new(mode);
    modulator.batch_process(&image, output_dir, base_name, image_config)
}

/// 获取处理特定图像和模式的预估内存使用量
///
/// # 参数
/// * `image_width` - 图像宽度
/// * `image_height` - 图像高度  
/// * `mode` - SSTV模式
/// * `sample_rate` - 采样率
///
/// # 返回
/// 预估的内存使用量（字节）
pub fn estimate_memory_usage(
    image_width: u32, 
    image_height: u32, 
    mode: SstvMode,
    sample_rate: u32
) -> usize {
    let (target_width, target_height) = mode.get_dimensions();
    let duration = mode.get_duration();
    
    // 原始图像内存（用于加载和预处理）
    let source_image_memory = (image_width * image_height * 3) as usize;
    
    // 目标图像内存：目标分辨率 * 3字节/像素
    let target_image_memory = (target_width * target_height * 3) as usize;
    
    // 音频内存：采样率 * 时长 * 2字节/样本
    let audio_memory = (sample_rate as f64 * duration * 2.0) as usize;
    
    // 处理过程中的峰值内存使用（原图+目标图+音频+开销）
    let peak_memory = source_image_memory + target_image_memory + audio_memory + 1024;
    
    peak_memory
}

/// 检查系统是否有足够内存处理指定的SSTV任务
///
/// # 参数
/// * `image_width` - 图像宽度
/// * `image_height` - 图像高度
/// * `mode` - SSTV模式
/// * `sample_rate` - 采样率
///
/// # 返回
/// (是否有足够内存, 需要的内存MB, 建议的最大图像尺寸)
pub fn check_memory_requirements(
    image_width: u32,
    image_height: u32, 
    mode: SstvMode,
    sample_rate: u32
) -> (bool, f64, Option<(u32, u32)>) {
    let required_memory = estimate_memory_usage(image_width, image_height, mode, sample_rate);
    let required_mb = required_memory as f64 / 1024.0 / 1024.0;
    
    // 假设可用内存为100MB（保守估计）
    let available_mb = 100.0;
    let has_enough = required_mb <= available_mb;
    
    let suggested_size = if !has_enough {
        // 计算建议的缩放比例，考虑原始图像尺寸
        let scale_factor = (available_mb / required_mb * 0.8).sqrt(); // 80%安全边际
        let new_width = ((image_width as f64 * scale_factor) as u32).max(100);
        let new_height = ((image_height as f64 * scale_factor) as u32).max(100);
        Some((new_width, new_height))
    } else {
        None
    };
    
    (has_enough, required_mb, suggested_size)
}

/// 一键完整处理：生成SSTV音频和保存处理后图片，带内存管理
///
/// # 参数
/// * `input_path` - 输入图像文件路径
/// * `output_dir` - 输出目录路径
/// * `base_name` - 基础文件名
/// * `mode` - SSTV模式
/// * `image_config` - 图片保存配置
/// * `memory_limit_mb` - 内存限制(MB)，None表示不限制
///
/// # 返回
/// (音频文件路径, 图片文件路径, 内存使用统计)
pub fn process_sstv_complete<P1, P2, P3>(
    input_path: P1,
    output_dir: P2,
    base_name: P3,
    mode: SstvMode,
    image_config: &ImageSaveConfig,
    memory_limit_mb: Option<usize>
) -> Result<(std::path::PathBuf, std::path::PathBuf, MemoryUsageMB)>
where
    P1: AsRef<std::path::Path>,
    P2: AsRef<std::path::Path>,
    P3: AsRef<str>,
{
    // 加载图像并检查内存需求
    let image = image::open(&input_path)
        .map_err(|e| SstvError::ImageProcessing(format!("无法加载图像: {}", e)))?;
    
    let (width, height) = (image.width(), image.height());
    
    // 内存检查
    if let Some(limit) = memory_limit_mb {
        let (has_enough, required_mb, _suggested_size) = check_memory_requirements(
            width, height, mode, DEFAULT_SAMPLE_RATE
        );
        
        if !has_enough && required_mb > limit as f64 {
            return Err(SstvError::MemoryError { 
                required: (required_mb * 1024.0 * 1024.0) as usize
            });
        }
    }
    
    // 创建调制器并处理
    let mut modulator = SstvModulator::new(mode);
    let (audio_path, image_path) = modulator.batch_process(
        &image, 
        output_dir, 
        base_name, 
        image_config
    )?;
    
    // 获取内存使用统计
    let memory_stats = modulator.get_memory_usage().to_mb();
    
    // 清理内存
    modulator.clear_memory();
    
    Ok((audio_path, image_path, memory_stats))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_supported_modes() {
        let modes = get_supported_modes();
        assert_eq!(modes.len(), 4);
        
        // 验证每个模式的基本信息
        for (mode, name, size, time) in modes {
            assert!(!name.is_empty());
            assert!(size.0 > 0 && size.1 > 0);
            assert!(time > 0.0);
            
            // 验证模式属性一致性
            assert_eq!(size, mode.get_dimensions());
            assert_eq!(time, mode.get_duration());
        }
    }

    #[test]
    fn test_file_size_estimation() {
        let size = estimate_file_size(SstvMode::Robot36, 44100, 16);
        assert!(size > 0);
        
        // Robot-36模式约36秒，44.1kHz采样率，16位
        let expected_samples = 36.0 * 44100.0;
        let expected_size = (expected_samples * 2.0) as usize + 44;
        assert!((size as f32 - expected_size as f32).abs() / expected_size as f32 < 0.1);
    }

    #[test]
    fn test_memory_estimation() {
        let usage = estimate_memory_usage(1920, 1080, SstvMode::Robot36, 44100);
        assert!(usage > 0);
        
        // Robot36: 320x240, ~36秒
        // 预期：原图内存 + 目标图内存 + 音频内存
        let expected_min = 320 * 240 * 3 + 44100 * 36 * 2; // 最小估算
        assert!(usage >= expected_min);
    }

    #[test]
    fn test_memory_requirements_check() {
        let (has_enough, required_mb, suggested) = check_memory_requirements(
            4000, 3000, SstvMode::Pd120, 44100
        );
        
        assert!(required_mb > 0.0);
        
        // 大图像应该建议缩小
        if !has_enough {
            assert!(suggested.is_some());
            let (w, h) = suggested.unwrap();
            assert!(w < 4000 && h < 3000);
        }
    }

    #[test]
    fn test_project_completeness() {
        // 验证所有必要的公共接口都存在
        let modes = get_supported_modes();
        assert_eq!(modes.len(), 4);
        
        // 验证默认配置
        let config = ImageSaveConfig::default();
        assert!(matches!(config.format, ImageFormat::Png));
        
        // 验证内存管理结构
        let usage = MemoryUsage {
            audio_samples_bytes: 1024,
            processed_image_bytes: 2048,
            metadata_bytes: 128,
            total_bytes: 3200,
        };
        let mb_usage = usage.to_mb();
        assert!(mb_usage.total_mb > 0.0);
    }
}