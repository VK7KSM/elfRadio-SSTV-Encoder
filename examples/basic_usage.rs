//! SSTV-Rust 完整批量处理示例
//! 
//! 将 test_image.jpg 一次性生成所有支持的SSTV音频文件和处理后的图片

use sstv_rust::{
    SstvModulator, SstvMode, ImageSaveConfig,
    generate_sstv_with_image_save
};
use std::path::Path;
use std::fs;

fn main() {
    println!("SSTV-Rust 完整批量处理工具");
    println!("============================");
    
    // 检查输入文件
    if !Path::new("test_image.jpg").exists() {
        println!("❌ 错误：未找到输入文件 'test_image.jpg'");
        println!("请将要处理的图片文件命名为 'test_image.jpg' 并放在程序目录下。");
        println!("支持的图片格式：JPG, PNG, BMP, TIFF 等");
        return;
    }
    
    // 创建输出目录
    if let Err(e) = fs::create_dir_all("media") {
        println!("❌ 错误：无法创建输出目录 'media': {}", e);
        return;
    }
    
    println!("✅ 找到输入文件：test_image.jpg");
    println!("✅ 输出目录：./media/");
    println!();
    
    // 定义处理参数
    let sample_rates = [6000, 16000, 44100];
    
    // 自定义采样率配置（默认被注释，用户可以取消注释启用）
    // 取消下面的注释可以启用自定义采样率功能
    // 支持范围：1000Hz - 192000Hz
    /*
    let custom_sample_rates = [
        8000,   // 电话质量
        11025,  // 低质量音频
        22050,  // FM广播质量  
        48000,  // 专业音频
        96000,  // 高分辨率音频
    ];
    */
    
    let modes = [
        (SstvMode::ScottieDx, "ScottieDX", "320x256"),
        (SstvMode::Robot36, "Robot36", "320x240"),
        (SstvMode::Pd120, "PD120", "640x496"),
        (SstvMode::MartinM1, "MartinM1", "320x256"),
    ];
    
    let mut success_count = 0;
    let mut total_files = sample_rates.len() * modes.len();
    
    // 如果启用了自定义采样率，更新文件总数
    // 取消下面的注释以支持自定义采样率统计
    /*
    total_files += custom_sample_rates.len() * modes.len();
    */
    
    println!("开始批量处理 ({} 个音频文件 + {} 个图片文件)：", total_files, modes.len());
    println!();
    
    // 为每种SSTV模式生成图片（只需要生成一次）
    for (mode, mode_name, resolution) in &modes {
        print!("🖼️  处理 {} 图片 ({})... ", mode_name, resolution);
        
        match process_image_for_mode(*mode, mode_name) {
            Ok(image_path) => {
                println!("✅ 已保存: {}", image_path.display());
                success_count += 1;
            }
            Err(e) => {
                println!("❌ 失败: {}", e);
            }
        }
    }
    
    println!();
    
    // 为标准采样率生成音频文件
        for &sample_rate in &sample_rates {
        println!("🎵 生成标准采样率 {}Hz 的音频文件:", sample_rate);
        
        for (mode, mode_name, resolution) in &modes {
            print!("   - {} ({})... ", mode_name, resolution);
            
            match process_audio_for_mode(*mode, mode_name, sample_rate) {
                Ok(audio_path) => {
                    println!("✅ {}", audio_path.display());
                    success_count += 1;
                }
                Err(e) => {
                    println!("❌ 失败: {}", e);
                }
            }
        }
        println!();
    }
    
    // 自定义采样率处理（默认被注释）
    // 取消下面的注释块可以启用自定义采样率功能
    /*
    println!("🎵 生成自定义采样率的音频文件:");
    for &sample_rate in &custom_sample_rates {
        println!("   采样率 {}Hz:", sample_rate);
        
        for (mode, mode_name, resolution) in &modes {
            print!("   - {} ({})... ", mode_name, resolution);
            
            match process_audio_for_mode(*mode, mode_name, sample_rate) {
                Ok(audio_path) => {
                    println!("✅ {}", audio_path.display());
                    success_count += 1;
                }
                Err(e) => {
                    println!("❌ 失败: {}", e);
                }
            }
        }
        println!();
    }
    */
    
    // 处理结果统计
    let total_expected = total_files + modes.len(); // 音频文件 + 图片文件
    println!("处理完成！");
    println!("成功: {}/{} 个文件", success_count, total_expected);
    
    if success_count == total_expected {
        println!("✅ 所有文件处理成功！");
        println!();
        println!("输出文件列表:");
        println!("📁 音频文件 ({}个):", total_files);
        
        // 显示标准采样率文件
        for &sample_rate in &sample_rates {
            for (_, mode_name, _) in &modes {
                println!("   - media/sstv_{}_{}_{}hz_16bit.wav", mode_name, get_timestamp(), sample_rate);
            }
        }
        
        // 显示自定义采样率文件（默认被注释）
        // 取消下面的注释可以显示自定义采样率文件列表
        /*
        for &sample_rate in &custom_sample_rates {
            for (_, mode_name, _) in &modes {
                println!("   - media/sstv_{}_{}_{}hz_16bit.wav", mode_name, get_timestamp(), sample_rate);
            }
        }
        */
        
        println!("📁 图片文件 ({}个):", modes.len());
        for (_, mode_name, resolution) in &modes {
            println!("   - media/sstv_{}_{}_{}.png", mode_name, get_timestamp(), resolution);
        }
        
        println!();
        println!("使用说明:");
        println!("• 音频文件可直接用于SSTV传输");
        println!("• 图片文件是经过SSTV标准处理的压缩图片");
        println!("• 标准采样率说明:");
        println!("  - 6000Hz: 低带宽环境，文件最小");
        println!("  - 16000Hz: 标准质量，推荐使用");
        println!("  - 44100Hz: 高质量音频，文件较大");
        
        // 显示自定义采样率说明（默认被注释）
        // 取消下面的注释可以显示自定义采样率说明
        /*
        println!("• 自定义采样率说明:");
        println!("  - 8000Hz: 电话质量，兼容性好");
        println!("  - 11025Hz: 低质量音频，节省空间");
        println!("  - 22050Hz: FM广播质量");
        println!("  - 48000Hz: 专业音频标准");
        println!("  - 96000Hz: 高分辨率音频，文件很大");
        println!("• 自定义功能使用方法:");
        println!("  1. 找到代码中被注释的 custom_sample_rates 数组");
        println!("  2. 取消相关注释块（共3处）");
        println!("  3. 可根据需要修改采样率数值（支持1000-192000Hz）");
        println!("  4. 重新编译运行即可生成自定义采样率音频");
        */
        
    } else {
        println!("⚠️  部分文件处理失败，请检查错误信息。");
    }
    
    println!();
    println!("📝 自定义采样率功能说明:");
    println!("   如需生成其他采样率的音频文件，请编辑本示例代码，");
    println!("   取消注释 'custom_sample_rates' 相关代码块（共3处），");
    println!("   然后重新编译运行即可。支持范围：1000-192000Hz");
}

/// 为指定的SSTV模式处理并保存图片
fn process_image_for_mode(mode: SstvMode, mode_name: &str) -> Result<std::path::PathBuf, String> {
    // 加载图像
    let image = image::open("test_image.jpg")
        .map_err(|e| format!("无法加载图片文件: {}", e))?;
    
    // 创建调制器（使用默认采样率进行图像处理）
    let mut modulator = SstvModulator::new(mode);
    
    // 调制图像（这会触发图像预处理）
    modulator.modulate_image(&image)
        .map_err(|e| format!("图像调制失败: {}", e))?;
    
    // 生成图片文件名
    let (width, height) = mode.get_dimensions();
    let timestamp = get_timestamp();
    let filename = format!("sstv_{}_{}_{}_{}x{}.png", 
                          mode_name, 
                          timestamp,
                          "processed",
                          width, 
                          height);
    let image_path = Path::new("media").join(filename);
    
    // 保存处理后的图片
    let config = ImageSaveConfig::png();
    modulator.save_processed_image_with_config(&image_path, &config)
        .map_err(|e| format!("图片保存失败: {}", e))?;
    
    // 清理内存
    drop(modulator);
    
    Ok(image_path)
}

/// 为指定的SSTV模式和采样率生成音频文件
fn process_audio_for_mode(mode: SstvMode, mode_name: &str, sample_rate: u32) -> Result<std::path::PathBuf, String> {
    // 验证采样率范围
    if sample_rate < 1000 || sample_rate > 192000 {
        return Err(format!("不支持的采样率: {}Hz，支持范围: 1000-192000Hz", sample_rate));
    }
    
    // 加载图像
    let image = image::open("test_image.jpg")
        .map_err(|e| format!("无法加载图片文件: {}", e))?;
    
    // 创建调制器并设置采样率
    let mut modulator = SstvModulator::new(mode).with_sample_rate(sample_rate);
    
    // 调制图像
    modulator.modulate_image(&image)
        .map_err(|e| format!("音频调制失败: {}", e))?;
    
    // 生成音频文件名
    let timestamp = get_timestamp();
    let filename = format!("sstv_{}_{}_{}hz_16bit.wav", 
                          mode_name, 
                          timestamp,
                          sample_rate);
    let audio_path = Path::new("media").join(filename);
    
    // 导出WAV文件
    modulator.export_wav(&audio_path)
        .map_err(|e| format!("音频文件保存失败: {}", e))?;
    
    // 清理内存
    modulator.clear_memory();
    
    Ok(audio_path)
}

/// 获取当前时间戳
fn get_timestamp() -> String {
    chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string()
}

/// 获取SSTV模式的分辨率字符串
fn get_mode_resolution(mode: SstvMode) -> String {
    let (width, height) = mode.get_dimensions();
    format!("{}x{}", width, height)
}