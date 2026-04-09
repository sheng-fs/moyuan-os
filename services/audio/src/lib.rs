//! 音频子系统
//!
//! 实现操作系统的音频功能，支持音频播放、录制和管理

#![no_std]
#![allow(unexpected_cfgs)]
#![feature(alloc_error_handler)]

extern crate alloc;

pub mod driver;
pub mod mixer;
pub mod codec;
pub mod playback;
pub mod capture;
pub mod format;
pub mod settings;

use alloc::vec::Vec;
use crate::driver::AudioDriver;
use crate::mixer::AudioMixer;
use crate::playback::AudioPlayer;
use crate::capture::AudioRecorder;
use crate::settings::AudioSettings;

/// 音频服务初始化
pub fn init() -> Result<(), crate::error::ServiceError> {
    // 初始化各模块
    driver::init()?;
    mixer::init()?;
    codec::init()?;
    playback::init()?;
    capture::init()?;
    format::init()?;
    settings::init()?;

    Ok(())
}

/// 音频服务清理
pub fn cleanup() {
    // 清理各模块
    settings::cleanup();
    format::cleanup();
    capture::cleanup();
    playback::cleanup();
    codec::cleanup();
    mixer::cleanup();
    driver::cleanup();
}

/// 音频服务
pub struct AudioService {
    driver: AudioDriver,
    mixer: AudioMixer,
    player: AudioPlayer,
    recorder: AudioRecorder,
    settings: AudioSettings,
}

impl AudioService {
    /// 创建音频服务
    pub fn new() -> Self {
        Self {
            driver: AudioDriver::new(),
            mixer: AudioMixer::new(),
            player: AudioPlayer::new(),
            recorder: AudioRecorder::new(),
            settings: AudioSettings::default(),
        }
    }

    /// 播放音频
    pub fn play(&mut self, data: &[u8], format: crate::format::AudioFormat) -> Result<(), crate::error::ServiceError> {
        self.player.play(data, format)
    }

    /// 录制音频
    pub fn record(&mut self, duration: u32) -> Result<Vec<u8>, crate::error::ServiceError> {
        self.recorder.record(duration)
    }

    /// 设置音量
    pub fn set_volume(&mut self, volume: f32) -> Result<(), crate::error::ServiceError> {
        self.mixer.set_volume(volume)
    }

    /// 获取音量
    pub fn get_volume(&self) -> f32 {
        self.mixer.get_volume()
    }

    /// 暂停播放
    pub fn pause(&mut self) -> Result<(), crate::error::ServiceError> {
        self.player.pause()
    }

    /// 恢复播放
    pub fn resume(&mut self) -> Result<(), crate::error::ServiceError> {
        self.player.resume()
    }

    /// 停止播放
    pub fn stop(&mut self) -> Result<(), crate::error::ServiceError> {
        self.player.stop()
    }

    /// 获取设置
    pub fn settings(&self) -> &AudioSettings {
        &self.settings
    }

    /// 更新设置
    pub fn update_settings(&mut self, settings: AudioSettings) {
        self.settings = settings;
    }
}

/// 错误模块
pub mod error {
    #[derive(Debug)]
    pub enum ServiceError {
        InitializationError,
        DriverError,
        PlaybackError,
        CaptureError,
        FormatError,
        SettingsError,
    }

    impl From<core::fmt::Error> for ServiceError {
        fn from(_: core::fmt::Error) -> Self {
            ServiceError::FormatError
        }
    }
}


