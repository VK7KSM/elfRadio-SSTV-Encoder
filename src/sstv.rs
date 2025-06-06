use crate::audio::{AudioProcessor, WavWriter};
use crate::error::SstvError;
use image::{DynamicImage, RgbImage, Rgb, ImageBuffer, ImageFormat};
use std::f64::consts::PI;
use std::path::Path;

// SSTV模式定义
#[derive(Debug, Clone, Copy)]
pub enum SstvMode {
    ScottieDx,  // 320x256, 269.6秒
    Robot36,    // 320x240, 36.0秒
    Pd120,      // 640x496, 120.0秒
    MartinM1,   // 320x256, 114.7秒
}

impl SstvMode {
    pub fn get_vis_code(&self) -> &'static str {
        match self {
            SstvMode::ScottieDx => "1001100",
            SstvMode::Robot36 => "0001000",
            SstvMode::Pd120 => "1011111",
            SstvMode::MartinM1 => "0101100",
        }
    }
    
    pub fn get_dimensions(&self) -> (u32, u32) {
        match self {
            SstvMode::ScottieDx => (320, 256),
            SstvMode::Robot36 => (320, 240),
            SstvMode::Pd120 => (640, 496),
            SstvMode::MartinM1 => (320, 256),
        }
    }
    
    pub fn get_duration(&self) -> f64 {
        match self {
            SstvMode::ScottieDx => 269.6,
            SstvMode::Robot36 => 36.0,
            SstvMode::Pd120 => 120.0,
            SstvMode::MartinM1 => 114.7,
        }
    }
    
    pub fn get_mode_name(&self) -> &'static str {
        match self {
            SstvMode::ScottieDx => "ScottieDX",
            SstvMode::Robot36 => "Robot36",
            SstvMode::Pd120 => "PD120", 
            SstvMode::MartinM1 => "MartinM1",
        }
    }
}

/// 图片保存格式配置
#[derive(Debug, Clone)]
pub struct ImageSaveConfig {
    /// 图片格式
    pub format: ImageFormat,
    /// JPEG质量 (1-100)
    pub jpeg_quality: Option<u8>,
    /// 是否保留元数据
    pub preserve_metadata: bool,
    /// 自定义后缀
    pub custom_suffix: Option<String>,
}

impl Default for ImageSaveConfig {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            jpeg_quality: Some(95),
            preserve_metadata: true,
            custom_suffix: None,
        }
    }
}

impl ImageSaveConfig {
    /// 创建PNG格式配置
    pub fn png() -> Self {
        Self {
            format: ImageFormat::Png,
            ..Default::default()
        }
    }
    
    /// 创建JPEG格式配置
    pub fn jpeg(quality: u8) -> Self {
        Self {
            format: ImageFormat::Jpeg,
            jpeg_quality: Some(quality.clamp(1, 100)),
            ..Default::default()
        }
    }
    
    /// 创建BMP格式配置
    pub fn bmp() -> Self {
        Self {
            format: ImageFormat::Bmp,
            ..Default::default()
        }
    }
    
    /// 设置自定义后缀
    pub fn with_suffix<S: Into<String>>(mut self, suffix: S) -> Self {
        self.custom_suffix = Some(suffix.into());
        self
    }
}

/// 处理信息元数据
#[derive(Debug, Clone)]
pub struct ProcessingMetadata {
    pub original_dimensions: (u32, u32),
    pub target_dimensions: (u32, u32),
    pub sstv_mode: SstvMode,
    pub scale_factor: f64,
    pub black_bars: (u32, u32, u32, u32), // left, top, right, bottom
    pub processing_timestamp: String,
}

/// 内存使用统计
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// 音频样本占用字节数
    pub audio_samples_bytes: usize,
    /// 处理后图像占用字节数
    pub processed_image_bytes: usize,
    /// 元数据占用字节数
    pub metadata_bytes: usize,
    /// 总占用字节数
    pub total_bytes: usize,
}

impl MemoryUsage {
    /// 转换为MB
    pub fn to_mb(&self) -> MemoryUsageMB {
        MemoryUsageMB {
            audio_samples_mb: self.audio_samples_bytes as f64 / 1024.0 / 1024.0,
            processed_image_mb: self.processed_image_bytes as f64 / 1024.0 / 1024.0,
            metadata_mb: self.metadata_bytes as f64 / 1024.0 / 1024.0,
            total_mb: self.total_bytes as f64 / 1024.0 / 1024.0,
        }
    }
}

/// 内存使用统计（MB单位）
#[derive(Debug, Clone)]
pub struct MemoryUsageMB {
    pub audio_samples_mb: f64,
    pub processed_image_mb: f64,
    pub metadata_mb: f64,
    pub total_mb: f64,
}

// SSTV调制器
pub struct SstvModulator {
    mode: SstvMode,
    sample_rate: u32,
    audio_processor: AudioProcessor,
    // 相位连续性变量
    older_data: f64,
    older_cos: f64,
    delta_length: f64,
    // 存储处理后的图像和元数据
    processed_image: Option<RgbImage>,
    processing_metadata: Option<ProcessingMetadata>,
}

impl SstvModulator {
    pub fn new(mode: SstvMode) -> Self {
        Self {
            mode,
            sample_rate: crate::DEFAULT_SAMPLE_RATE,  // 使用6000Hz优化采样率
            audio_processor: AudioProcessor::new(crate::DEFAULT_SAMPLE_RATE),
            older_data: 0.0,
            older_cos: 1.0,
            delta_length: 0.0,
            processed_image: None,
            processing_metadata: None,
        }
    }
    
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self.audio_processor = AudioProcessor::new(sample_rate);
        self
    }
    
    /// 主要的图像调制方法 - 包含智能图片预处理
    pub fn modulate_image(&mut self, image: &DynamicImage) -> Result<Vec<i16>, SstvError> {
        // 智能图像预处理：保持宽高比，填充黑边
        let (rgb_image, metadata) = self.preprocess_image_with_aspect_ratio(image)?;
        
        // 存储处理后的图像和元数据
        self.processed_image = Some(rgb_image.clone());
        self.processing_metadata = Some(metadata);
        
        // 重置音频处理器和相位连续性变量
        self.audio_processor.clear();
        self.older_data = 0.0;
        self.older_cos = 1.0;
        self.delta_length = 0.0;
        
        // 添加开始静音
        self.write_tone(0.0, 200.0)?;
        
        // 生成VIS码
        self.generate_vis_code()?;
        
        // 根据模式生成SSTV信号
        match self.mode {
            SstvMode::ScottieDx => self.generate_scottie_dx(&rgb_image)?,
            SstvMode::Robot36 => self.generate_robot36(&rgb_image)?,
            SstvMode::Pd120 => self.generate_pd120(&rgb_image)?,
            SstvMode::MartinM1 => self.generate_martin_m1(&rgb_image)?,
        }
        
        // 生成结束音
        self.generate_end_tones()?;
        
        // 添加结束静音
        self.write_tone(0.0, 200.0)?;
        
        Ok(self.audio_processor.get_samples().to_vec())
    }
    
    /// 智能图像预处理：保持宽高比并填充黑边（带元数据记录和内存优化）
    fn preprocess_image_with_aspect_ratio(&self, image: &DynamicImage) -> Result<(RgbImage, ProcessingMetadata), SstvError> {
        let (target_width, target_height) = self.mode.get_dimensions();
        let (src_width, src_height) = (image.width(), image.height());
        
        // 检查原图大小，如果过大则提前警告
        let source_pixels = src_width as u64 * src_height as u64;
        let target_pixels = target_width as u64 * target_height as u64;
        
        if source_pixels > target_pixels * 16 {
            eprintln!("警告：原图像过大 ({}x{})，建议预先缩小以节省内存", src_width, src_height);
        }
        
        // 计算缩放比例，保持宽高比
        let scale_x = target_width as f64 / src_width as f64;
        let scale_y = target_height as f64 / src_height as f64;
        let scale = scale_x.min(scale_y); // 使用较小的比例以确保图像完全适合
        
        // 计算缩放后的尺寸
        let scaled_width = (src_width as f64 * scale) as u32;
        let scaled_height = (src_height as f64 * scale) as u32;
        
        // 缩放图像
        let scaled_image = image.resize(
            scaled_width, 
            scaled_height, 
            image::imageops::FilterType::Lanczos3
        );
        
        // 创建目标尺寸的黑色背景图像
        let mut target_image = ImageBuffer::from_pixel(
            target_width, 
            target_height, 
            Rgb([0, 0, 0]) // 黑色背景
        );
        
        // 计算居中位置和黑边信息
        let offset_x = (target_width - scaled_width) / 2;
        let offset_y = (target_height - scaled_height) / 2;
        
        // 将缩放后的图像复制到目标图像的中心
        let scaled_rgb = scaled_image.to_rgb8();
        for y in 0..scaled_height {
            for x in 0..scaled_width {
                let pixel = scaled_rgb.get_pixel(x, y);
                target_image.put_pixel(offset_x + x, offset_y + y, *pixel);
            }
        }
        
        // 创建处理元数据
        let metadata = ProcessingMetadata {
            original_dimensions: (src_width, src_height),
            target_dimensions: (target_width, target_height),
            sstv_mode: self.mode,
            scale_factor: scale,
            black_bars: (
                offset_x, 
                offset_y, 
                target_width - offset_x - scaled_width, 
                target_height - offset_y - scaled_height
            ),
            processing_timestamp: chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string(),
        };
        
        Ok((target_image, metadata))
    }
    
    /// 保存处理后的图像（基础方法）
    pub fn save_processed_image<P: AsRef<Path>>(&self, path: P) -> Result<(), SstvError> {
        self.save_processed_image_with_config(path, &ImageSaveConfig::default())
    }
    
    /// 保存处理后的图像（高级配置）
    pub fn save_processed_image_with_config<P: AsRef<Path>>(
        &self, 
        path: P, 
        config: &ImageSaveConfig
    ) -> Result<(), SstvError> {
        let image = self.processed_image.as_ref()
            .ok_or_else(|| SstvError::ImageProcessing("没有处理后的图像可保存，请先调用 modulate_image".to_string()))?;
        
        let path = path.as_ref();
        
        // 根据配置保存图像
        match config.format {
            ImageFormat::Png => {
                image.save_with_format(path, ImageFormat::Png)
                    .map_err(|e| SstvError::ImageProcessing(format!("PNG保存失败: {}", e)))?;
            },
            ImageFormat::Jpeg => {
                // 创建JPEG编码器以设置质量
                let mut buffer = Vec::new();
                {
                    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                        &mut buffer, 
                        config.jpeg_quality.unwrap_or(95)
                    );
                    encoder.encode(
                        image.as_raw(),
                        image.width(),
                        image.height(),
                        image::ColorType::Rgb8.into()
                    ).map_err(|e| SstvError::ImageProcessing(format!("JPEG编码失败: {}", e)))?;
                }
                std::fs::write(path, buffer)
                    .map_err(|e| SstvError::IoError(e))?;
            },
            ImageFormat::Bmp => {
                image.save_with_format(path, ImageFormat::Bmp)
                    .map_err(|e| SstvError::ImageProcessing(format!("BMP保存失败: {}", e)))?;
            },
            _ => {
                return Err(SstvError::ImageProcessing(format!("不支持的图像格式: {:?}", config.format)));
            }
        }
        
        // 如果需要保留元数据，保存元数据文件
        if config.preserve_metadata {
            self.save_metadata_file(path)?;
        }
        
        Ok(())
    }
    
    /// 自动生成文件名并保存
    pub fn save_processed_image_auto<P: AsRef<Path>>(
        &self, 
        base_dir: P, 
        config: &ImageSaveConfig
    ) -> Result<std::path::PathBuf, SstvError> {
        let base_dir = base_dir.as_ref();
        
        // 创建目录（如果不存在）
        if !base_dir.exists() {
            std::fs::create_dir_all(base_dir)
                .map_err(|e| SstvError::IoError(e))?;
        }
        
        // 生成文件名
        let timestamp = self.processing_metadata.as_ref()
            .map(|m| m.processing_timestamp.clone())
            .unwrap_or_else(|| chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string());
        
        let mode_name = self.mode.get_mode_name();
        let (width, height) = self.mode.get_dimensions();
        
        let suffix = config.custom_suffix.as_ref()
            .map(|s| format!("_{}", s))
            .unwrap_or_default();
        
        let extension = match config.format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg", 
            ImageFormat::Bmp => "bmp",
            _ => "png",
        };
        
        let filename = format!(
            "sstv_{}_{}_{}x{}{}.{}", 
            mode_name, timestamp, width, height, suffix, extension
        );
        
        let full_path = base_dir.join(filename);
        
        // 保存图像
        self.save_processed_image_with_config(&full_path, config)?;
        
        Ok(full_path)
    }
    
    /// 保存处理元数据到JSON文件
    fn save_metadata_file<P: AsRef<Path>>(&self, image_path: P) -> Result<(), SstvError> {
        let metadata = self.processing_metadata.as_ref()
            .ok_or_else(|| SstvError::ImageProcessing("没有处理元数据".to_string()))?;
        
        let image_path = image_path.as_ref();
        let metadata_path = image_path.with_extension("json");
        
        let metadata_json = serde_json::json!({
            "sstv_processing_info": {
                "version": crate::VERSION,
                "sstv_mode": metadata.sstv_mode.get_mode_name(),
                "original_dimensions": {
                    "width": metadata.original_dimensions.0,
                    "height": metadata.original_dimensions.1
                },
                "target_dimensions": {
                    "width": metadata.target_dimensions.0,
                    "height": metadata.target_dimensions.1
                },
                "scale_factor": metadata.scale_factor,
                "black_bars": {
                    "left": metadata.black_bars.0,
                    "top": metadata.black_bars.1,
                    "right": metadata.black_bars.2,
                    "bottom": metadata.black_bars.3
                },
                "processing_timestamp": metadata.processing_timestamp,
                "sample_rate": self.sample_rate,
                "duration_seconds": metadata.sstv_mode.get_duration()
            }
        });
        
        std::fs::write(metadata_path, serde_json::to_string_pretty(&metadata_json)
            .map_err(|e| SstvError::ImageProcessing(format!("JSON序列化失败: {}", e)))?)
            .map_err(|e| SstvError::IoError(e))?;
        
        Ok(())
    }
    
    /// 批处理：同时生成音频和保存图像
    pub fn batch_process<P1, P2>(
        &mut self,
        input_image: &DynamicImage,
        output_dir: P1, 
        base_name: P2,
        image_config: &ImageSaveConfig
    ) -> Result<(std::path::PathBuf, std::path::PathBuf), SstvError>
    where
        P1: AsRef<Path>,
        P2: AsRef<str>,
    {
        let output_dir = output_dir.as_ref();
        let base_name = base_name.as_ref();
        
        // 创建输出目录
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)
                .map_err(|e| SstvError::IoError(e))?;
        }
        
        // 调制图像
        let _samples = self.modulate_image(input_image)?;
        
        // 生成文件路径
        let mode_name = self.mode.get_mode_name();
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        
        let audio_filename = format!("{}_{}_{}_{}.wav", base_name, mode_name, timestamp, self.sample_rate);
        let audio_path = output_dir.join(audio_filename);
        
        // 保存音频
        self.export_wav(&audio_path)?;
        
        // 保存图像（使用自动命名）
        let mut image_config = image_config.clone();
        image_config.custom_suffix = Some(base_name.to_string());
        let image_path = self.save_processed_image_auto(output_dir, &image_config)?;
        
        Ok((audio_path, image_path))
    }
    
    /// 获取处理后的图像（用于进一步处理）
    pub fn get_processed_image(&self) -> Option<&RgbImage> {
        self.processed_image.as_ref()
    }
    
    /// 获取处理元数据
    pub fn get_processing_metadata(&self) -> Option<&ProcessingMetadata> {
        self.processing_metadata.as_ref()
    }
    
    /// 清理内存中的数据（避免内存泄漏）
    pub fn clear_memory(&mut self) {
        self.audio_processor.clear();
        self.processed_image = None;
        self.processing_metadata = None;
        self.older_data = 0.0;
        self.older_cos = 1.0;
        self.delta_length = 0.0;
    }
    
    /// 分阶段清理音频内存
    pub fn clear_audio_memory(&mut self) {
        self.audio_processor.clear();
        self.older_data = 0.0;
        self.older_cos = 1.0;
        self.delta_length = 0.0;
    }
    
    /// 分阶段清理图像内存
    pub fn clear_image_memory(&mut self) {
        self.processed_image = None;
        self.processing_metadata = None;
    }
    
    /// 获取当前内存使用统计
    pub fn get_memory_usage(&self) -> MemoryUsage {
        let audio_samples_bytes = self.audio_processor.get_samples().len() * std::mem::size_of::<i16>();
        let image_bytes = self.processed_image.as_ref()
            .map(|img| img.width() * img.height() * 3) // RGB = 3 bytes per pixel
            .unwrap_or(0) as usize;
        
        MemoryUsage {
            audio_samples_bytes,
            processed_image_bytes: image_bytes,
            metadata_bytes: std::mem::size_of::<ProcessingMetadata>(),
            total_bytes: audio_samples_bytes + image_bytes + std::mem::size_of::<ProcessingMetadata>(),
        }
    }
    
    /// 强制进行垃圾回收提示
    pub fn force_gc_hint(&self) {
        // 在处理大文件后建议系统进行垃圾回收
        // Rust没有显式GC，但我们可以释放不必要的容量
    }
    
    /// 检查是否需要内存清理（基于使用量）
    pub fn should_clear_memory(&self, threshold_mb: usize) -> bool {
        let usage = self.get_memory_usage();
        usage.total_bytes > threshold_mb * 1024 * 1024
    }
    
    /// 智能内存管理：根据阈值自动清理
    pub fn auto_memory_management(&mut self, threshold_mb: usize) {
        if self.should_clear_memory(threshold_mb) {
            self.clear_memory();
        }
    }
    
    fn generate_vis_code(&mut self) -> Result<(), SstvError> {
        let vis_code = self.mode.get_vis_code();
        
        // 前导音序列
        let preamble_tones = [
            (1900.0, 100.0), (1500.0, 100.0), (1900.0, 100.0), (1500.0, 100.0),
            (2300.0, 100.0), (1500.0, 100.0), (2300.0, 100.0), (1500.0, 100.0),
            // VIS码引导音
            (1900.0, 300.0), (1200.0, 10.0), (1900.0, 300.0), (1200.0, 30.0),
        ];
        
        for (freq, duration) in &preamble_tones {
            self.write_tone(*freq, *duration)?;
        }
        
        // VIS码7位数据位（小端序，从第6位到第0位）
        for i in (0..7).rev() {
            let bit = vis_code.chars().nth(i).unwrap();
            let frequency = if bit == '1' { 1100.0 } else { 1300.0 };
            self.write_tone(frequency, 30.0)?;
        }
        
        // 偶校验位
        let ones_count = vis_code.chars().filter(|&c| c == '1').count();
        let parity_freq = if ones_count % 2 == 0 { 1300.0 } else { 1100.0 };
        self.write_tone(parity_freq, 30.0)?;
        
        // 结束位
        self.write_tone(1200.0, 30.0)?;
        
        Ok(())
    }
    
    fn generate_scottie_dx(&mut self, image: &RgbImage) -> Result<(), SstvError> {
        let (width, height) = image.dimensions();
        
        // 起始同步脉冲，仅第一行（严格按照PDF中的C代码）
        self.write_tone_with_phase(1200.0, 9.0, 0.0)?;
        
        // 图像数据部分
        for row in 0..height {
            // 分离脉冲（使用相位连续性）
            self.write_tone_with_continuous_phase(1500.0, 1.5)?;
            
            // 绿色扫描
            for col in 0..width {
                let pixel = image.get_pixel(col, row);
                let green_freq = 1500.0 + (pixel[1] as f64) * COLOR_FREQ_MULT;
                self.write_tone_with_continuous_phase(green_freq, 1.08)?;
            }
            
            // 分离脉冲（使用相位连续性）
            self.write_tone_with_continuous_phase(1500.0, 1.5)?;
            
            // 蓝色扫描
            for col in 0..width {
                let pixel = image.get_pixel(col, row);
                let blue_freq = 1500.0 + (pixel[2] as f64) * COLOR_FREQ_MULT;
                self.write_tone_with_continuous_phase(blue_freq, 1.08)?;
            }
            
            // 同步脉冲与同步沿（使用相位连续性）
            self.write_tone_with_continuous_phase(1200.0, 9.0)?;
            self.write_tone_with_continuous_phase(1500.0, 1.5)?;
            
            // 红色扫描
            for col in 0..width {
                let pixel = image.get_pixel(col, row);
                let red_freq = 1500.0 + (pixel[0] as f64) * COLOR_FREQ_MULT;
                self.write_tone_with_continuous_phase(red_freq, 1.08)?;
            }
        }
        
        Ok(())
    }
    
    fn generate_robot36(&mut self, image: &RgbImage) -> Result<(), SstvError> {
        let (width, height) = image.dimensions();
        
        for row in 0..height {
            // 同步脉冲
            self.write_tone(1200.0, 9.0)?;
            // Porch脉冲
            self.write_tone(1500.0, 3.0)?;
            
            if row % 2 == 0 {
                // 偶数行亮度扫描
                for col in 0..width {
                    let y_value = self.get_y_value(image, col, row);
                    let y_freq = 1500.0 + y_value * COLOR_FREQ_MULT;
                    self.write_tone(y_freq, 0.275)?;
                }
                
                // 偶数分离脉冲
                self.write_tone(1500.0, 4.5)?;
                // Porch脉冲
                self.write_tone(1900.0, 1.5)?;
                
                // 两行RY均值扫描
                for col in 0..width {
                    let ry1 = self.get_ry_value(image, col, row);
                    let ry2 = if row + 1 < height {
                        self.get_ry_value(image, col, row + 1)
                    } else {
                        ry1
                    };
                    let ry_avg = (ry1 + ry2) / 2.0;
                    let ry_freq = 1500.0 + ry_avg * COLOR_FREQ_MULT;
                    self.write_tone(ry_freq, 0.1375)?;
                }
            } else {
                // 奇数行亮度扫描
                for col in 0..width {
                    let y_value = self.get_y_value(image, col, row);
                    let y_freq = 1500.0 + y_value * COLOR_FREQ_MULT;
                    self.write_tone(y_freq, 0.275)?;
                }
                
                // 奇数分离脉冲
                self.write_tone(2300.0, 4.5)?;
                // Porch脉冲
                self.write_tone(1900.0, 1.5)?;
                
                // 两行BY均值扫描
                for col in 0..width {
                    let by1 = self.get_by_value(image, col, row);
                    let by2 = if row + 1 < height {
                        self.get_by_value(image, col, row + 1)
                    } else {
                        by1
                    };
                    let by_avg = (by1 + by2) / 2.0;
                    let by_freq = 1500.0 + by_avg * COLOR_FREQ_MULT;
                    self.write_tone(by_freq, 0.1375)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_pd120(&mut self, image: &RgbImage) -> Result<(), SstvError> {
        let (width, height) = image.dimensions();
        
        for row in (0..height).step_by(2) {
            // 长同步脉冲
            self.write_tone(1200.0, 20.0)?;
            // Porch脉冲
            self.write_tone(1500.0, 2.08)?;
            
            // 偶数行亮度扫描
            for col in 0..width {
                let y_value = self.get_y_value(image, col, row);
                let y_freq = 1500.0 + y_value * COLOR_FREQ_MULT;
                self.write_tone(y_freq, 0.19)?;
            }
            
            // 两行RY均值扫描
            for col in 0..width {
                let ry1 = self.get_ry_value(image, col, row);
                let ry2 = if row + 1 < height {
                    self.get_ry_value(image, col, row + 1)
                } else {
                    ry1
                };
                let ry_avg = (ry1 + ry2) / 2.0;
                let ry_freq = 1500.0 + ry_avg * COLOR_FREQ_MULT;
                self.write_tone(ry_freq, 0.19)?;
            }
            
            // 两行BY均值扫描
            for col in 0..width {
                let by1 = self.get_by_value(image, col, row);
                let by2 = if row + 1 < height {
                    self.get_by_value(image, col, row + 1)
                } else {
                    by1
                };
                let by_avg = (by1 + by2) / 2.0;
                let by_freq = 1500.0 + by_avg * COLOR_FREQ_MULT;
                self.write_tone(by_freq, 0.19)?;
            }
            
            // 奇数行亮度扫描
            if row + 1 < height {
                for col in 0..width {
                    let y_value = self.get_y_value(image, col, row + 1);
                    let y_freq = 1500.0 + y_value * COLOR_FREQ_MULT;
                    self.write_tone(y_freq, 0.19)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn generate_martin_m1(&mut self, image: &RgbImage) -> Result<(), SstvError> {
        let (width, height) = image.dimensions();
        
        for row in 0..height {
            // Martin M1标准时序参数（基于参考实现）
            // 总时长：114.7秒，分辨率：320x256
            // 扫描顺序：GBR（绿色-蓝色-红色）
            // 每像素时间：457.6微秒
            // 同步脉冲：1200Hz 4.862ms
            // 颜色分隔符：1500Hz 0.572ms
            
            // 同步脉冲 + 颜色分隔符
            self.write_tone(1200.0, 4.862)?;  // 同步脉冲
            self.write_tone(1500.0, 0.572)?;  // 颜色分隔符
            
            // GBR扫描顺序：绿色-蓝色-红色（与参考代码一致）
            for color_index in [1, 2, 0] {  // GBR顺序：绿色(1)-蓝色(2)-红色(0)
                // 扫描当前颜色通道的所有像素
                for col in 0..width {
                    let pixel = image.get_pixel(col, row);
                    let color_value = pixel[color_index] as f64;
                    let freq = 1500.0 + color_value * COLOR_FREQ_MULT;
                    self.write_tone(freq, 0.4576)?;  // 457.6微秒
                }
                
                // 颜色通道之间的分隔符
                self.write_tone(1500.0, 0.572)?;  // 颜色分隔符
            }
        }
        
        Ok(())
    }
    
    fn generate_end_tones(&mut self) -> Result<(), SstvError> {
        let end_tones = [
            (1500.0, 500.0),
            (1900.0, 100.0),
            (1500.0, 100.0),
            (1900.0, 100.0),
            (1500.0, 100.0),
        ];
        
        for (freq, duration) in &end_tones {
            self.write_tone(*freq, *duration)?;
        }
        
        Ok(())
    }
    
    // YUV颜色空间转换函数（与C实现完全一致）
    fn get_y_value(&self, image: &RgbImage, x: u32, y: u32) -> f64 {
        let pixel = image.get_pixel(x, y);
        let r = pixel[0] as f64;
        let g = pixel[1] as f64;
        let b = pixel[2] as f64;
        
        16.0 + (0.003906 * ((65.738 * r) + (129.057 * g) + (25.064 * b)))
    }
    
    fn get_ry_value(&self, image: &RgbImage, x: u32, y: u32) -> f64 {
        let pixel = image.get_pixel(x, y);
        let r = pixel[0] as f64;
        let g = pixel[1] as f64;
        let b = pixel[2] as f64;
        
        128.0 + (0.003906 * ((112.439 * r) + (-94.154 * g) + (-18.285 * b)))
    }
    
    fn get_by_value(&self, image: &RgbImage, x: u32, y: u32) -> f64 {
        let pixel = image.get_pixel(x, y);
        let r = pixel[0] as f64;
        let g = pixel[1] as f64;
        let b = pixel[2] as f64;
        
        128.0 + (0.003906 * ((-37.945 * r) + (-74.494 * g) + (112.439 * b)))
    }
    
    // 写入音调，严格按照PDF文章中的C代码实现相位连续性算法
    fn write_tone(&mut self, frequency: f64, duration_ms: f64) -> Result<(), SstvError> {
        // 计算样本数（与C代码完全一致）
        let mut num_samples = ((self.sample_rate as f64) * duration_ms / 1000.0) as u32;
        
        // 累积误差补偿（与C代码完全一致）
        self.delta_length += (self.sample_rate as f64) * duration_ms / 1000.0 - (num_samples as f64);
        if self.delta_length >= 1.0 {
            num_samples += self.delta_length as u32;
            self.delta_length -= self.delta_length.floor();
        }
        
        // 计算相位连续性的初始相位（严格按照PDF中的C代码）
        let sign_older_cos = if self.older_cos >= 0.0 { 1.0_f64 } else { -1.0_f64 };
        let abs_sign_diff = (sign_older_cos - 1.0_f64).abs() / 2.0_f64;
        let phi = sign_older_cos * self.older_data.asin() + abs_sign_diff * PI;
        
        // 生成音频样本（修正相位计算）
        for i in 0..num_samples {
            let phase = 2.0 * PI * frequency * (i as f64) / (self.sample_rate as f64) + phi;
            let sample_value = phase.sin();
            let sample = (32767.0 * sample_value) as i16;
            self.audio_processor.add_sample(sample);
        }
        
        // 更新相位连续性变量（修正相位计算）
        let final_phase = 2.0 * PI * frequency * (num_samples as f64) / (self.sample_rate as f64) + phi;
        self.older_data = final_phase.sin();
        self.older_cos = final_phase.cos();
        
        Ok(())
    }
    
    // 带指定相位的音调写入函数
    fn write_tone_with_phase(&mut self, frequency: f64, duration_ms: f64, phi: f64) -> Result<(), SstvError> {
        // 计算样本数
        let mut num_samples = ((self.sample_rate as f64) * duration_ms / 1000.0) as u32;
        
        // 累积误差补偿
        self.delta_length += (self.sample_rate as f64) * duration_ms / 1000.0 - (num_samples as f64);
        if self.delta_length >= 1.0 {
            num_samples += self.delta_length as u32;
            self.delta_length -= self.delta_length.floor();
        }
        
        // 生成音频样本（修正相位计算）
        for i in 0..num_samples {
            let phase = 2.0 * PI * frequency * (i as f64) / (self.sample_rate as f64) + phi;
            let sample_value = phase.sin();
            let sample = (32767.0 * sample_value) as i16;
            self.audio_processor.add_sample(sample);
        }
        
        // 更新相位连续性变量（修正相位计算）
        let final_phase = 2.0 * PI * frequency * (num_samples as f64) / (self.sample_rate as f64) + phi;
        self.older_data = final_phase.sin();
        self.older_cos = final_phase.cos();
        
        Ok(())
    }
    
    // 使用相位连续性的音调写入函数（严格按照PDF中的公式）
    fn write_tone_with_continuous_phase(&mut self, frequency: f64, duration_ms: f64) -> Result<(), SstvError> {
        // 计算相位连续性的相位（严格按照PDF中的公式）
        let sign_older_cos = if self.older_cos >= 0.0 { 1.0_f64 } else { -1.0_f64 };
        let abs_sign_diff = (sign_older_cos - 1.0_f64).abs() / 2.0_f64;
        let phi = sign_older_cos * self.older_data.asin() + abs_sign_diff * PI;
        
        self.write_tone_with_phase(frequency, duration_ms, phi)
    }
    
    pub fn export_wav<P: AsRef<Path>>(&self, filename: P) -> Result<(), SstvError> {
        let mut writer = WavWriter::new(filename, self.sample_rate)?;
        writer.write_samples(self.audio_processor.get_samples())?;
        writer.finalize()?;
        Ok(())
    }
    
    pub fn get_samples(&self) -> &[i16] {
        self.audio_processor.get_samples()
    }
    
    pub fn get_mode(&self) -> SstvMode {
        self.mode
    }
    
    pub fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

// 颜色频率乘数常量（与C实现完全一致）
const COLOR_FREQ_MULT: f64 = 3.1372549;

impl Drop for SstvModulator {
    fn drop(&mut self) {
        // 确保在对象销毁时清理所有内存
        self.clear_memory();
    }
}