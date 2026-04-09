//! 输入法服务
//!
//! 实现操作系统的输入法功能，支持多语言输入

#![no_std]
#![allow(unexpected_cfgs)]
#![feature(alloc_error_handler)]

extern crate alloc;

pub mod engine;
pub mod ime;
pub mod keyboard;
pub mod candidates;
pub mod settings;

use alloc::string::String;
use crate::engine::InputEngine;
use crate::ime::InputMethodEditor;
use crate::keyboard::KeyboardLayout;
use crate::candidates::CandidateList;
use crate::settings::InputSettings;

/// 输入法服务初始化
pub fn init() -> Result<(), crate::error::ServiceError> {
    // 初始化各模块
    engine::init()?;
    ime::init()?;
    keyboard::init()?;
    candidates::init()?;
    settings::init()?;

    Ok(())
}

/// 输入法服务清理
pub fn cleanup() {
    // 清理各模块
    settings::cleanup();
    candidates::cleanup();
    keyboard::cleanup();
    ime::cleanup();
    engine::cleanup();
}

/// 输入法服务
pub struct InputMethodService {
    engine: InputEngine,
    ime: InputMethodEditor,
    keyboard_layout: KeyboardLayout,
    candidate_list: CandidateList,
    settings: InputSettings,
}

impl InputMethodService {
    /// 创建输入法服务
    pub fn new() -> Self {
        Self {
            engine: InputEngine::new(),
            ime: InputMethodEditor::new(),
            keyboard_layout: KeyboardLayout::default(),
            candidate_list: CandidateList::new(),
            settings: InputSettings::default(),
        }
    }

    /// 处理键盘输入
    pub fn handle_key_input(&mut self, key: u32, modifiers: u32) -> Result<Option<String>, crate::error::ServiceError> {
        self.engine.process_key(key, modifiers)
    }

    /// 获取候选词列表
    pub fn get_candidates(&self) -> &CandidateList {
        &self.candidate_list
    }

    /// 选择候选词
    pub fn select_candidate(&mut self, index: usize) -> Result<String, crate::error::ServiceError> {
        self.ime.select_candidate(index)
    }

    /// 设置输入法
    pub fn set_input_method(&mut self, ime_name: &str) -> Result<(), crate::error::ServiceError> {
        self.engine.set_input_method(ime_name)
    }

    /// 设置键盘布局
    pub fn set_keyboard_layout(&mut self, layout: KeyboardLayout) {
        self.keyboard_layout = layout;
    }

    /// 获取设置
    pub fn settings(&self) -> &InputSettings {
        &self.settings
    }

    /// 更新设置
    pub fn update_settings(&mut self, settings: InputSettings) {
        self.settings = settings;
    }
}

/// 错误模块
pub mod error {
    #[derive(Debug)]
    pub enum ServiceError {
        InitializationError,
        InputError,
        SettingsError,
        ImeNotFound,
        LayoutNotFound,
    }

    impl From<core::fmt::Error> for ServiceError {
        fn from(_: core::fmt::Error) -> Self {
            ServiceError::InputError
        }
    }
}


