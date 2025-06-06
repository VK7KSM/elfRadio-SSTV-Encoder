[English Version](README.md) | 中文版本

# elfRadio-SSTV-Encoder 超高性能SSTV编码器

我在用Rust编写《电台精灵》项目时，发现Rust根本找不到好用的SSTV编码库，所以只能动手编写自己的SSTV编码库，然后《elfRadio-SSTV-Encoder》就诞生了。这是基于无线电爱好者们贡献的优化算法，实现了任意格式图片到SSTV任意采样率音频文件的高效转换，支持6kHz低采样率优化。生成的6kHz采样率音频文件可被RX-SSTV等软件正常解码，且解码后的图片质量与44.1kHz采样率音频格式解码效果相同。

## 核心功能

### 🎯 SSTV编码功能
- **4种标准SSTV模式**：Scottie-DX、Robot-36、PD-120、Martin-M1
- **智能图片预处理**：自动调整分辨率，保持宽高比，黑边填充
- **处理后图片保存**：输出符合SSTV标准的压缩图片
- **多采样率支持**：6kHz优化、16kHz标准、44.1kHz高质量，以及任意自定义采样率
- **内存管理优化**：避免内存泄漏，支持批量处理

### 🚀 性能优化
- **6000Hz采样率优化**：基于奈奎斯特定理，文件大小减少86.4%
- **相位连续性算法**：确保音频信号的连续性和质量
- **采样精度补偿**：delta_length算法保证精确时序
- **高效音频生成**：优化的VIS码和扫描线生成算法

## 快速开始

### 安装依赖

首先确保已安装Rust工具链：
```bash
# 安装Rust（如果尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 运行示例程序

1. **准备测试图片**：
   ```bash
   # 将您的图片重命名为 test_image.jpg 并放在项目根目录
   cp your_image.jpg test_image.jpg
   ```

2. **运行批量处理示例**：
   ```bash
   cargo run --example basic_usage
   ```

3. **生成的文件**：
   ```
   media/
   ├── sstv_ScottieDX_20241220_143022_6000hz_16bit.wav
   ├── sstv_Robot36_20241220_143022_6000hz_16bit.wav
   ├── sstv_PD120_20241220_143022_6000hz_16bit.wav
   ├── sstv_MartinM1_20241220_143022_6000hz_16bit.wav
   ├── (更多采样率的音频文件...)
   ├── sstv_ScottieDX_20241220_143022_processed_320x256.png
   ├── sstv_Robot36_20241220_143022_processed_320x240.png
   ├── sstv_PD120_20241220_143022_processed_640x496.png
   └── sstv_MartinM1_20241220_143022_processed_320x256.png
   ```

## 编程接口使用

### 基本用法

```rust
use sstv_rust::{SstvModulator, SstvMode};
use image::open;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载图片
    let image = open("input.jpg")?;
    
    // 创建Robot-36模式的调制器
    let mut modulator = SstvModulator::new(SstvMode::Robot36);
    
    // 调制图片生成音频
    modulator.modulate_image(&image)?;
    
    // 导出音频文件
    modulator.export_wav("output.wav")?;
    
    // 保存处理后的图片
    modulator.save_processed_image("processed.png")?;
    
    // 清理内存
    modulator.clear_memory();
    
    Ok(())
}
```

### 自定义采样率

```rust
use sstv_rust::{SstvModulator, SstvMode};

// 创建6kHz采样率的调制器（文件最小）
let mut modulator = SstvModulator::new(SstvMode::ScottieDx)
    .with_sample_rate(6000);

// 创建44.1kHz采样率的调制器（音质最高）
let mut modulator = SstvModulator::new(SstvMode::Pd120)
    .with_sample_rate(44100);

// 创建自定义采样率的调制器
let mut modulator = SstvModulator::new(SstvMode::MartinM1)
    .with_sample_rate(22050);
```

### 批量处理

```rust
use sstv_rust::{SstvModulator, SstvMode, ImageSaveConfig};
use std::path::Path;

fn batch_process() -> Result<(), Box<dyn std::error::Error>> {
    let modes = [SstvMode::Robot36, SstvMode::ScottieDx];
    let sample_rates = [6000, 16000, 44100];
    
    for mode in &modes {
        for &sample_rate in &sample_rates {
            // 加载图片
            let image = image::open("input.jpg")?;
            
            // 创建调制器
            let mut modulator = SstvModulator::new(*mode)
                .with_sample_rate(sample_rate);
            
            // 生成音频文件
            modulator.modulate_image(&image)?;
            let audio_filename = format!("output_{}_{}.wav", 
                                       format!("{:?}", mode).to_lowercase(), 
                                       sample_rate);
            modulator.export_wav(&audio_filename)?;
            
            // 生成图片文件（每种模式只需要保存一次）
            if sample_rate == sample_rates[0] {
                let image_filename = format!("processed_{}.png", 
                                            format!("{:?}", mode).to_lowercase());
                modulator.save_processed_image(&image_filename)?;
            }
            
            // 清理内存
            modulator.clear_memory();
        }
    }
    
    Ok(())
}
```

### 高级配置

```rust
use sstv_rust::{SstvModulator, SstvMode, ImageSaveConfig};

fn advanced_usage() -> Result<(), Box<dyn std::error::Error>> {
    let image = image::open("input.jpg")?;
    let mut modulator = SstvModulator::new(SstvMode::Pd120);
    
    // 调制图片
    modulator.modulate_image(&image)?;
    
    // 导出音频
    modulator.export_wav("output.wav")?;
    
    // 配置图片保存选项
    let config = ImageSaveConfig::png().with_compression(9);
    modulator.save_processed_image_with_config("output.png", &config)?;
    
    // 获取处理信息
    println!("音频样本数: {}", modulator.get_samples().len());
    println!("采样率: {}Hz", modulator.get_sample_rate());
    println!("模式: {:?}", modulator.get_mode());
    
    // 获取内存使用情况
    let memory_usage = modulator.get_memory_usage();
    println!("内存使用: 音频{}KB, 图片{}KB", 
             memory_usage.audio_samples_bytes / 1024,
             memory_usage.processed_image_bytes / 1024);
    
    Ok(())
}
```

## 集成到其他项目

### Cargo.toml 配置

```toml
[dependencies]
sstv-rust = { path = "path/to/sstv-rust" }
image = "0.25.6"
```

### 作为库使用

```rust
// 在您的项目中
use sstv_rust::{SstvModulator, SstvMode, generate_sstv_with_image_save};

// 方式1：使用便捷函数
let (audio_path, image_path) = generate_sstv_with_image_save(
    "input.jpg",
    "output",
    "my_sstv",
    SstvMode::Robot36,
    &ImageSaveConfig::png()
)?;

// 方式2：集成到音频处理流水线
pub struct AudioProcessor {
    sstv_modulator: SstvModulator,
}

impl AudioProcessor {
    pub fn new() -> Self {
        Self {
            sstv_modulator: SstvModulator::new(SstvMode::Robot36),
        }
    }
    
    pub fn process_image(&mut self, image_data: &[u8]) -> Result<Vec<i16>, Error> {
        let image = image::load_from_memory(image_data)?;
        let samples = self.sstv_modulator.modulate_image(&image)?;
        Ok(samples)
    }
}
```

### Web服务集成示例

```rust
// 基于axum的Web API示例
use axum::{extract::Multipart, response::Json, routing::post, Router};
use sstv_rust::{SstvModulator, SstvMode};

async fn upload_and_convert(mut multipart: Multipart) -> Result<Json<ApiResponse>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("image") {
            let data = field.bytes().await?;
            let image = image::load_from_memory(&data)?;
            
            let mut modulator = SstvModulator::new(SstvMode::Robot36);
            modulator.modulate_image(&image)?;
            
            // 保存文件并返回URL
            let filename = format!("sstv_{}.wav", uuid::Uuid::new_v4());
            modulator.export_wav(&format!("uploads/{}", filename))?;
            
            return Ok(Json(ApiResponse {
                success: true,
                audio_url: format!("/downloads/{}", filename),
            }));
        }
    }
    
    Err(ApiError::MissingImage)
}
```

## 支持的SSTV模式详细信息

| 模式 | 分辨率 | 传输时间 | 文件大小@6kHz | 适用场景 |
|------|--------|----------|---------------|----------|
| **Robot-36** | 320×240 | 36.0秒 | ~432KB | 快速传输，适合实时通信 |
| **Scottie-DX** | 320×256 | 269.6秒 | ~3.2MB | 高质量图像，业余无线电常用 |
| **Martin-M1** | 320×256 | 114.7秒 | ~1.4MB | 平衡速度和质量 |
| **PD-120** | 640×496 | 120.0秒 | ~1.4MB | 高分辨率，适合详细图像 |

## 采样率选择指南

### 推荐采样率
- **6000Hz**：文件最小，编码速度极快，适合边缘设备和开发板。
- **16000Hz**：标准质量，通用推荐，适合直接解码音频文件
- **44100Hz**：最高质量，适合存档

### 自定义采样率
- **支持范围**：6000Hz - 192000Hz
- **计算公式**：文件大小 ≈ 采样率 × 传输时间 × 2字节
- **质量考虑**：SSTV频率范围1500-2300Hz，采样率≥5000Hz即可保证质量

## 图片预处理说明

库会自动对输入图片进行智能预处理：

1. **尺寸调整**：自动调整到目标SSTV模式的分辨率
2. **宽高比保持**：保持原图宽高比，避免变形
3. **黑边填充**：空白区域用黑色填充
4. **颜色优化**：根据SSTV模式进行RGB/YUV转换

## API参考

### SstvModulator

```rust
impl SstvModulator {
    // 构造函数
    pub fn new(mode: SstvMode) -> Self
    pub fn with_sample_rate(self, sample_rate: u32) -> Self
    
    // 核心功能
    pub fn modulate_image(&mut self, image: &DynamicImage) -> Result<Vec<i16>>
    pub fn export_wav<P: AsRef<Path>>(&self, path: P) -> Result<()>
    pub fn save_processed_image<P: AsRef<Path>>(&self, path: P) -> Result<()>
    
    // 配置选项
    pub fn save_processed_image_with_config<P: AsRef<Path>>(
        &self, path: P, config: &ImageSaveConfig
    ) -> Result<()>
    
    // 信息获取
    pub fn get_samples(&self) -> &[i16]
    pub fn get_mode(&self) -> SstvMode
    pub fn get_sample_rate(&self) -> u32
    pub fn get_memory_usage(&self) -> MemoryUsage
    
    // 内存管理
    pub fn clear_memory(&mut self)
    pub fn clear_audio_samples(&mut self)
    pub fn clear_image_memory(&mut self)
}
```

### 便捷函数

```rust
// 生成SSTV音频和图片
pub fn generate_sstv_with_image_save<P1, P2, P3>(
    input_path: P1,
    output_dir: P2, 
    base_name: P3,
    mode: SstvMode,
    config: &ImageSaveConfig
) -> Result<(PathBuf, PathBuf)>

// 批量处理
pub fn process_sstv_complete<P1, P2, P3>(
    input_path: P1,
    output_dir: P2,
    base_name: P3, 
    mode: SstvMode,
    image_config: &ImageSaveConfig
) -> Result<(PathBuf, PathBuf)>
```

## 性能基准

在现代硬件上的典型性能（Intel i7, 16GB RAM）：

| 测试项目 | 耗时 | 处理速度 |
|----------|------|----------|
| 1920×1080图片预处理 | ~5ms | 实时 |
| Robot-36音频生成 | ~15ms | 2400x实时 |
| Scottie-DX音频生成 | ~50ms | 5392x实时 |
| WAV文件导出 | ~2ms | 即时 |
| 内存清理 | ~1ms | 即时 |

## 故障排除

### 常见问题

**Q: 生成的音频文件无法被SSTV软件解码？**
A: 解码方式分为直接解码音频文件和解码器通过声卡采集声音信号后解码。如果是直接解码，必须生成16000Hz以上采样率的音频文件；如果是用于采集声音信号后解码，则6kHz的采样率音频文件即可。

**Q: 处理大图片时内存占用过高？**
A: 使用`clear_memory()`方法及时清理，或分批处理图片。

**Q: 自定义采样率不起作用？**
A: 确保采样率在6000-192000Hz范围内，且为整数。

**Q: 输出图片质量不理想？**
A: 检查输入图片分辨率，选择合适的SSTV模式。

### 调试技巧

```rust
// 启用详细日志
let mut modulator = SstvModulator::new(SstvMode::Robot36);
let memory_before = modulator.get_memory_usage();
modulator.modulate_image(&image)?;
let memory_after = modulator.get_memory_usage();
println!("内存变化: {}KB", 
         (memory_after.total_bytes - memory_before.total_bytes) / 1024);
```

## 贡献和许可

### 开源许可
本项目采用MIT许可证，可自由使用、修改和分发，无需通知作者。

### 特别感谢
感谢以下无线电爱好者和前辈们的无私贡献：
- **BG7ZDQ** - 核心算法优化和技术指导
- **BG2TFM** - 相位连续性算法改进
- **BI4PYM** - 采样精度补偿算法
- **所有业余无线电爱好者** - 多年来积累的SSTV技术经验和代码贡献

没有你们的智慧和分享精神，就不会有这个高性能的Rust实现。

### 项目愿景
希望这个库能够：
- 推广SSTV技术在现代软件中的应用
- 为业余无线电爱好者提供高效的工具
- 促进无线电技术与现代编程语言的结合
- 传承和发展无线电技术文化

### 相关资源
- [业余无线电SSTV规范](http://www.barberdsp.com/files/Dayton%20Paper.pdf)
- [RX-SSTV解码软件](http://users.belgacom.net/hamradio/rxsstv.htm)
- [MMSSTV发射软件](https://hamsoft.ca/pages/mmsstv.php)
- [业余无线电协会](http://www.crac.org.cn/)

---

**让我们一起传承和发展业余无线电技术！ 73!**