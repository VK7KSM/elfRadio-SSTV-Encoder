//! 音频处理和WAV文件生成模块
//! 
//! 本模块提供音频信号生成和WAV文件输出功能。

use crate::error::{Result, SstvError};
use hound::{WavSpec, WavWriter as HoundWavWriter};
use std::path::Path;

/// 音频生成器
pub struct AudioGenerator {
    sample_rate: u32,
    bit_depth: u16,
}

impl AudioGenerator {
    /// 创建新的音频生成器
    pub fn new(sample_rate: u32, bit_depth: u16) -> Result<Self> {
        if sample_rate < 8000 || sample_rate > 192000 {
            return Err(SstvError::invalid_sample_rate(sample_rate, 8000, 192000));
        }

        if bit_depth != 16 && bit_depth != 24 && bit_depth != 32 {
            return Err(SstvError::InvalidAudioParameter {
                parameter: "bit_depth".to_string(),
                value: bit_depth.to_string(),
            });
        }

        Ok(Self {
            sample_rate,
            bit_depth,
        })
    }

    /// 生成正弦波信号
    pub fn generate_sine_wave(&self, frequency: f32, duration: f32, amplitude: f32) -> Vec<f32> {
        let num_samples = (duration * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f32 / self.sample_rate as f32;
            let sample = amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin();
            samples.push(sample);
        }

        samples
    }

    /// 生成线性调频信号（chirp）
    pub fn generate_chirp(&self, start_freq: f32, end_freq: f32, duration: f32, amplitude: f32) -> Vec<f32> {
        let num_samples = (duration * self.sample_rate as f32) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f32 / self.sample_rate as f32;
            let normalized_time = t / duration;
            let instantaneous_freq = start_freq + (end_freq - start_freq) * normalized_time;
            let phase = 2.0 * std::f32::consts::PI * instantaneous_freq * t;
            let sample = amplitude * phase.sin();
            samples.push(sample);
        }

        samples
    }

    /// 应用窗函数（汉宁窗）
    pub fn apply_hanning_window(&self, samples: &mut [f32]) {
        let len = samples.len();
        for (i, sample) in samples.iter_mut().enumerate() {
            let window_value = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (len - 1) as f32).cos());
            *sample *= window_value;
        }
    }

    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// 获取位深度
    pub fn bit_depth(&self) -> u16 {
        self.bit_depth
    }
}

/// 音频处理器 - 用于收集和处理音频样本
pub struct AudioProcessor {
    samples: Vec<i16>,
    sample_rate: u32,
}

impl AudioProcessor {
    /// 创建新的音频处理器
    pub fn new(sample_rate: u32) -> Self {
        Self {
            samples: Vec::new(),
            sample_rate,
        }
    }

    /// 添加音频样本
    pub fn add_sample(&mut self, sample: i16) {
        self.samples.push(sample);
    }

    /// 获取所有样本
    pub fn get_samples(&self) -> &[i16] {
        &self.samples
    }

    /// 清空样本
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

/// WAV文件写入器
pub struct WavWriter {
    spec: WavSpec,
    writer: Option<HoundWavWriter<std::io::BufWriter<std::fs::File>>>,
}

impl WavWriter {
    /// 创建新的WAV写入器
    pub fn new<P: AsRef<Path>>(filename: P, sample_rate: u32) -> Result<Self> {
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let writer = HoundWavWriter::create(filename, spec)?;

        Ok(Self {
            spec,
            writer: Some(writer),
        })
    }

    /// 创建用于SSTV的标准WAV写入器（单声道，16位）
    pub fn for_sstv<P: AsRef<Path>>(filename: P, sample_rate: u32) -> Result<Self> {
        Self::new(filename, sample_rate)
    }

    /// 写入i16样本
    pub fn write_samples(&mut self, samples: &[i16]) -> Result<()> {
        if let Some(ref mut writer) = self.writer {
            for &sample in samples {
                writer.write_sample(sample)?;
            }
        }
        Ok(())
    }

    /// 将浮点音频样本写入WAV文件
    pub fn write_samples_f32<P: AsRef<Path>>(path: P, samples: &[f32], sample_rate: u32) -> Result<()> {
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = HoundWavWriter::create(path, spec)?;

        // 将浮点样本转换为16位整数
        for &sample in samples {
            let sample_i16 = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(sample_i16)?;
        }

        writer.finalize()?;
        Ok(())
    }

    /// 完成写入并关闭文件
    pub fn finalize(&mut self) -> Result<()> {
        if let Some(writer) = self.writer.take() {
            writer.finalize()?;
        }
        Ok(())
    }

    /// 获取WAV规格
    pub fn spec(&self) -> &WavSpec {
        &self.spec
    }
}

/// 音频处理工具函数
pub mod utils {
    /// 将分贝转换为线性幅度
    pub fn db_to_linear(db: f32) -> f32 {
        10.0_f32.powf(db / 20.0)
    }

    /// 将线性幅度转换为分贝
    pub fn linear_to_db(linear: f32) -> f32 {
        20.0 * linear.log10()
    }

    /// 计算RMS值
    pub fn calculate_rms(samples: &[f32]) -> f32 {
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// 归一化音频样本
    pub fn normalize(samples: &mut [f32], target_peak: f32) {
        let max_amplitude = samples.iter().map(|&x| x.abs()).fold(0.0, f32::max);
        if max_amplitude > 0.0 {
            let scale_factor = target_peak / max_amplitude;
            for sample in samples.iter_mut() {
                *sample *= scale_factor;
            }
        }
    }
}

/// 音频效果处理模块
pub mod effects {
    /// 应用音量调整
    pub fn apply_volume(samples: &mut [f32], volume: f32) {
        for sample in samples.iter_mut() {
            *sample *= volume;
        }
    }

    /// 应用淡入效果
    pub fn apply_fade_in(samples: &mut [f32], fade_samples: usize) {
        let fade_samples = fade_samples.min(samples.len());
        for (i, sample) in samples.iter_mut().take(fade_samples).enumerate() {
            let factor = i as f32 / fade_samples as f32;
            *sample *= factor;
        }
    }

    /// 应用淡出效果
    pub fn apply_fade_out(samples: &mut [f32], fade_samples: usize) {
        let fade_samples = fade_samples.min(samples.len());
        let start_idx = samples.len().saturating_sub(fade_samples);
        
        for (i, sample) in samples.iter_mut().skip(start_idx).enumerate() {
            let factor = 1.0 - (i as f32 / fade_samples as f32);
            *sample *= factor;
        }
    }

    /// 应用低通滤波器（简单的一阶滤波器）
    pub fn apply_lowpass_filter(samples: &mut [f32], cutoff_ratio: f32) {
        if samples.is_empty() || cutoff_ratio >= 1.0 {
            return;
        }

        let alpha = cutoff_ratio.clamp(0.0, 1.0);
        let mut prev_sample = samples[0];

        for sample in samples.iter_mut().skip(1) {
            *sample = alpha * *sample + (1.0 - alpha) * prev_sample;
            prev_sample = *sample;
        }
    }

    /// 应用高通滤波器
    pub fn apply_highpass_filter(samples: &mut [f32], cutoff_ratio: f32) {
        if samples.is_empty() || cutoff_ratio <= 0.0 {
            return;
        }

        let alpha = (1.0 - cutoff_ratio).clamp(0.0, 1.0);
        let mut prev_input = samples[0];
        let mut prev_output = samples[0];

        for sample in samples.iter_mut().skip(1) {
            let current_input = *sample;
            *sample = alpha * (prev_output + current_input - prev_input);
            prev_input = current_input;
            prev_output = *sample;
        }
    }

    /// 应用带通滤波器
    pub fn apply_bandpass_filter(samples: &mut [f32], low_cutoff: f32, high_cutoff: f32) {
        // 先应用高通滤波器
        apply_highpass_filter(samples, low_cutoff);
        // 再应用低通滤波器
        apply_lowpass_filter(samples, high_cutoff);
    }
}

/// 从WAV文件加载音频数据
pub fn load_wav_file<P: AsRef<Path>>(path: P) -> Result<(Vec<f32>, u32)> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    
    let samples: std::result::Result<Vec<f32>, hound::Error> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().collect()
        },
        hound::SampleFormat::Int => {
            match spec.bits_per_sample {
                16 => {
                    let samples: std::result::Result<Vec<i16>, hound::Error> = reader.samples::<i16>().collect();
                    samples.map(|samples| {
                        samples.into_iter()
                            .map(|s| s as f32 / i16::MAX as f32)
                            .collect()
                    })
                },
                32 => {
                    let samples: std::result::Result<Vec<i32>, hound::Error> = reader.samples::<i32>().collect();
                    samples.map(|samples| {
                        samples.into_iter()
                            .map(|s| s as f32 / i32::MAX as f32)
                            .collect()
                    })
                },
                _ => return Err(SstvError::InvalidFormat(format!("不支持的位深度: {}", spec.bits_per_sample))),
            }
        },
    };
    
    let samples = samples.map_err(|e| SstvError::AudioError(e))?;
    Ok((samples, spec.sample_rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_generator_creation() {
        let generator = AudioGenerator::new(48000, 16).unwrap();
        assert_eq!(generator.sample_rate(), 48000);
        assert_eq!(generator.bit_depth(), 16);
    }

    #[test]
    fn test_sine_wave_generation() {
        let generator = AudioGenerator::new(48000, 16).unwrap();
        let samples = generator.generate_sine_wave(1000.0, 0.1, 0.5);
        assert_eq!(samples.len(), 4800); // 0.1s * 48000 samples/s
    }

    #[test]
    fn test_wav_writer_creation() {
        let writer = WavWriter::for_sstv("test.wav", 48000).unwrap();
        assert_eq!(writer.spec().sample_rate, 48000);
        assert_eq!(writer.spec().channels, 1);
        assert_eq!(writer.spec().bits_per_sample, 16);
    }

    #[test]
    fn test_db_conversion() {
        use utils::*;
        assert!((db_to_linear(0.0) - 1.0).abs() < 1e-6);
        assert!((linear_to_db(1.0) - 0.0).abs() < 1e-6);
    }
}