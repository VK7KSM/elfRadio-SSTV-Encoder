[English Version](README.md) | [中文版本](README_zh.md)

# elfRadio-SSTV-Encoder Ultra-High Performance SSTV Encoder

While developing the "elfRadio" project in Rust, I discovered there wasn't a decent SSTV encoding library available for Rust. This led me to create my own SSTV encoding library, and thus "elfRadio-SSTV-Encoder" was born. Built on optimization algorithms contributed by radio enthusiasts, this library efficiently converts images of any format into SSTV audio files at any sampling rate, with special optimization for 6kHz low sampling rates. The generated 6kHz audio files can be properly decoded by RX-SSTV and other software, producing the same image quality as 44.1kHz audio files.

## Core Features

### 🎯 SSTV Encoding Capabilities
- **4 Standard SSTV Modes**: Scottie-DX, Robot-36, PD-120, Martin-M1
- **Smart Image Preprocessing**: Automatic resolution adjustment with aspect ratio preservation and letterboxing
- **Processed Image Export**: Outputs SSTV-compliant compressed images
- **Flexible Sampling Rates**: 6kHz optimization, 16kHz standard, 44.1kHz high quality, plus any custom rate
- **Memory Management**: Prevents memory leaks with support for batch processing

### 🚀 Performance Optimizations
- **6000Hz Sampling Rate Optimization**: Based on Nyquist theorem, reduces file size by 86.4%
- **Phase Continuity Algorithm**: Ensures seamless audio signal continuity and quality
- **Sampling Precision Compensation**: delta_length algorithm guarantees precise timing
- **Efficient Audio Generation**: Optimized VIS code and scan line generation

## Getting Started

### Prerequisites

Make sure you have the Rust toolchain installed:
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Running the Example

1. **Prepare a test image**:
   ```bash
   # Rename your image to test_image.jpg and place it in the project root
   cp your_image.jpg test_image.jpg
   ```

2. **Run the batch processing example**:
   ```bash
   cargo run --example basic_usage
   ```

3. **Output files**:
   ```
   media/
   ├── sstv_ScottieDX_20241220_143022_6000hz_16bit.wav
   ├── sstv_Robot36_20241220_143022_6000hz_16bit.wav
   ├── sstv_PD120_20241220_143022_6000hz_16bit.wav
   ├── sstv_MartinM1_20241220_143022_6000hz_16bit.wav
   ├── (additional sampling rate audio files...)
   ├── sstv_ScottieDX_20241220_143022_processed_320x256.png
   ├── sstv_Robot36_20241220_143022_processed_320x240.png
   ├── sstv_PD120_20241220_143022_processed_640x496.png
   └── sstv_MartinM1_20241220_143022_processed_320x256.png
   ```

## Programming Interface

### Basic Usage

```rust
use sstv_rust::{SstvModulator, SstvMode};
use image::open;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the image
let image = open("input.jpg")?;

    // Create a Robot-36 mode modulator
let mut modulator = SstvModulator::new(SstvMode::Robot36);
    
    // Convert image to audio
    modulator.modulate_image(&image)?;
    
    // Export audio file
modulator.export_wav("output.wav")?;
    
    // Save the processed image
    modulator.save_processed_image("processed.png")?;
    
    // Clean up memory
    modulator.clear_memory();
    
    Ok(())
}
```

### Custom Sampling Rates

```rust
use sstv_rust::{SstvModulator, SstvMode};

// Create 6kHz modulator (smallest files)
let mut modulator = SstvModulator::new(SstvMode::ScottieDx)
    .with_sample_rate(6000);

// Create 44.1kHz modulator (highest quality)
let mut modulator = SstvModulator::new(SstvMode::Pd120)
    .with_sample_rate(44100);

// Create custom sampling rate modulator
let mut modulator = SstvModulator::new(SstvMode::MartinM1)
    .with_sample_rate(22050);
```

### Batch Processing

```rust
use sstv_rust::{SstvModulator, SstvMode, ImageSaveConfig};
use std::path::Path;

fn batch_process() -> Result<(), Box<dyn std::error::Error>> {
    let modes = [SstvMode::Robot36, SstvMode::ScottieDx];
    let sample_rates = [6000, 16000, 44100];
    
    for mode in &modes {
        for &sample_rate in &sample_rates {
            // Load image
let image = image::open("input.jpg")?;
            
            // Create modulator
            let mut modulator = SstvModulator::new(*mode)
                .with_sample_rate(sample_rate);
            
            // Generate audio file
            modulator.modulate_image(&image)?;
            let audio_filename = format!("output_{}_{}.wav", 
                                       format!("{:?}", mode).to_lowercase(), 
                                       sample_rate);
            modulator.export_wav(&audio_filename)?;
            
            // Save image file (once per mode)
            if sample_rate == sample_rates[0] {
                let image_filename = format!("processed_{}.png", 
                                            format!("{:?}", mode).to_lowercase());
                modulator.save_processed_image(&image_filename)?;
            }
            
            // Clean up
            modulator.clear_memory();
        }
    }
    
    Ok(())
}
```

### Advanced Configuration

```rust
use sstv_rust::{SstvModulator, SstvMode, ImageSaveConfig};

fn advanced_usage() -> Result<(), Box<dyn std::error::Error>> {
    let image = image::open("input.jpg")?;
    let mut modulator = SstvModulator::new(SstvMode::Pd120);
    
    // Process the image
    modulator.modulate_image(&image)?;
    
    // Export audio
    modulator.export_wav("output.wav")?;
    
    // Configure image save options
    let config = ImageSaveConfig::png().with_compression(9);
    modulator.save_processed_image_with_config("output.png", &config)?;
    
    // Get processing info
    println!("Audio samples: {}", modulator.get_samples().len());
    println!("Sample rate: {}Hz", modulator.get_sample_rate());
    println!("Mode: {:?}", modulator.get_mode());
    
    // Check memory usage
    let memory_usage = modulator.get_memory_usage();
    println!("Memory usage: Audio {}KB, Image {}KB", 
             memory_usage.audio_samples_bytes / 1024,
             memory_usage.processed_image_bytes / 1024);
    
    Ok(())
}
```

## Integrating into Your Project

### Cargo.toml Setup

```toml
[dependencies]
sstv-rust = { path = "path/to/sstv-rust" }
image = "0.25.6"
```

### Library Integration

```rust
// In your project
use sstv_rust::{SstvModulator, SstvMode, generate_sstv_with_image_save};

// Option 1: Use convenience functions
let (audio_path, image_path) = generate_sstv_with_image_save(
    "input.jpg",
    "output",
    "my_sstv",
    SstvMode::Robot36,
    &ImageSaveConfig::png()
)?;

// Option 2: Integrate into audio processing pipeline
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

### Web Service Integration

```rust
// Example using axum web framework
use axum::{extract::Multipart, response::Json, routing::post, Router};
use sstv_rust::{SstvModulator, SstvMode};

async fn upload_and_convert(mut multipart: Multipart) -> Result<Json<ApiResponse>, ApiError> {
    while let Some(field) = multipart.next_field().await? {
        if field.name() == Some("image") {
            let data = field.bytes().await?;
            let image = image::load_from_memory(&data)?;
            
            let mut modulator = SstvModulator::new(SstvMode::Robot36);
            modulator.modulate_image(&image)?;
            
            // Save and return file URL
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

## SSTV Mode Specifications

| Mode | Resolution | Transmission Time | File Size@6kHz | Best For |
|------|-----------|------------------|---------------|----------|
| **Robot-36** | 320×240 | 36.0s | ~432KB | Quick transmission, real-time communication |
| **Scottie-DX** | 320×256 | 269.6s | ~3.2MB | High quality images, popular in amateur radio |
| **Martin-M1** | 320×256 | 114.7s | ~1.4MB | Good balance of speed and quality |
| **PD-120** | 640×496 | 120.0s | ~1.4MB | High resolution for detailed images |

## Choosing Sampling Rates

### Recommended Rates
- **6000Hz**: Smallest files, blazing fast encoding, perfect for embedded systems and development boards
- **16000Hz**: Standard quality, recommended for most applications, ideal for direct audio file decoding
- **44100Hz**: Highest quality, best for archival purposes

### Custom Sampling Rates
- **Supported Range**: 6000Hz - 192000Hz
- **File Size Formula**: File Size ≈ Sample Rate × Transmission Time × 2 bytes
- **Quality Notes**: SSTV uses 1500-2300Hz frequency range, so ≥5000Hz sampling ensures perfect quality

## Image Preprocessing

The library automatically handles image preprocessing:

1. **Resolution Scaling**: Automatically resizes to match the target SSTV mode
2. **Aspect Ratio Preservation**: Maintains original proportions to prevent distortion
3. **Letterboxing**: Fills empty areas with black borders
4. **Color Space Optimization**: Converts between RGB/YUV based on the SSTV mode

## API Documentation

### SstvModulator

```rust
impl SstvModulator {
    // Construction
    pub fn new(mode: SstvMode) -> Self
    pub fn with_sample_rate(self, sample_rate: u32) -> Self
    
    // Core functionality
    pub fn modulate_image(&mut self, image: &DynamicImage) -> Result<Vec<i16>>
    pub fn export_wav<P: AsRef<Path>>(&self, path: P) -> Result<()>
    pub fn save_processed_image<P: AsRef<Path>>(&self, path: P) -> Result<()>
    
    // Advanced options
    pub fn save_processed_image_with_config<P: AsRef<Path>>(
        &self, path: P, config: &ImageSaveConfig
    ) -> Result<()>
    
    // Information access
    pub fn get_samples(&self) -> &[i16]
    pub fn get_mode(&self) -> SstvMode
    pub fn get_sample_rate(&self) -> u32
    pub fn get_memory_usage(&self) -> MemoryUsage
    
    // Memory management
    pub fn clear_memory(&mut self)
    pub fn clear_audio_samples(&mut self)
    pub fn clear_image_memory(&mut self)
}
```

### Convenience Functions

```rust
// Generate both SSTV audio and processed image
pub fn generate_sstv_with_image_save<P1, P2, P3>(
    input_path: P1,
    output_dir: P2, 
    base_name: P3,
    mode: SstvMode,
    config: &ImageSaveConfig
) -> Result<(PathBuf, PathBuf)>

// Complete batch processing
pub fn process_sstv_complete<P1, P2, P3>(
    input_path: P1,
    output_dir: P2,
    base_name: P3, 
    mode: SstvMode,
    image_config: &ImageSaveConfig
) -> Result<(PathBuf, PathBuf)>
```

## Performance Benchmarks

Performance on modern hardware (Intel i7, 16GB RAM):

| Operation | Time | Processing Speed |
|-----------|------|------------------|
| 1920×1080 image preprocessing | ~5ms | Real-time |
| Robot-36 audio generation | ~15ms | 2400x real-time |
| Scottie-DX audio generation | ~50ms | 5392x real-time |
| WAV file export | ~2ms | Instant |
| Memory cleanup | ~1ms | Instant |

## Troubleshooting

### Common Issues

**Q: Generated audio files won't decode in SSTV software?**
A: There are two decoding methods: direct audio file decoding and sound card signal capture. For direct decoding, use sampling rates ≥16000Hz. For sound card capture, 6kHz files work fine.

**Q: High memory usage with large images?**
A: Use `clear_memory()` regularly or process images in smaller batches.

**Q: Custom sampling rate not working?**
A: Ensure the rate is between 6000-192000Hz and is a whole number.

**Q: Poor output image quality?**
A: Check your input image resolution and choose an appropriate SSTV mode.

### Debugging Tips

```rust
// Monitor memory usage
let mut modulator = SstvModulator::new(SstvMode::Robot36);
let memory_before = modulator.get_memory_usage();
modulator.modulate_image(&image)?;
let memory_after = modulator.get_memory_usage();
println!("Memory delta: {}KB", 
         (memory_after.total_bytes - memory_before.total_bytes) / 1024);
```

## Contributing & License

### Open Source License
This project is released under the MIT license. Feel free to use, modify, and distribute without notification.

### Special Recognition
Huge thanks to these radio enthusiasts and mentors for their invaluable contributions:
- **BG7ZDQ** - Core algorithm optimization and technical guidance
- **BG2TFM** - Phase continuity algorithm enhancements
- **BI4PYM** - Sampling precision compensation algorithm
- **All amateur radio enthusiasts** - Years of accumulated SSTV expertise and code contributions

Without your knowledge and spirit of sharing, this high-performance Rust implementation wouldn't exist.

### Project Goals
This library aims to:
- Bring SSTV technology into modern software development
- Provide powerful tools for amateur radio enthusiasts
- Bridge classic radio techniques with contemporary programming languages
- Preserve and advance radio technology heritage

### Resources
- [Amateur Radio SSTV Standards](http://www.barberdsp.com/files/Dayton%20Paper.pdf)
- [RX-SSTV Decoder Software](http://users.belgacom.net/hamradio/rxsstv.htm)
- [MMSSTV Transmitter Software](https://hamsoft.ca/pages/mmsstv.php)

---

**Let's keep amateur radio technology alive and thriving! 73!**
