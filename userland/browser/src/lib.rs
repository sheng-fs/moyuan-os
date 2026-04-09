//! 浏览器
//!
//! 实现操作系统的网页浏览器功能，支持网页浏览、HTML 渲染和网络访问

#![no_std]
#![allow(unexpected_cfgs)]
#![feature(alloc_error_handler)]

extern crate alloc;

pub mod core;
pub mod renderer;
pub mod network;
pub mod storage;
pub mod ui;
pub mod javascript;
pub mod extensions;
pub mod settings;

use crate::core::BrowserCore;
use crate::renderer::HtmlRenderer;
use crate::network::NetworkManager;
use crate::storage::StorageManager;
use crate::ui::BrowserUI;
use crate::settings::BrowserSettings;

/// 浏览器初始化
pub fn init() -> Result<(), crate::error::BrowserError> {
    // 初始化各模块
    core::init()?;
    network::init()?;
    storage::init()?;
    renderer::init()?;
    javascript::init()?;
    ui::init()?;
    extensions::init()?;
    settings::init()?;

    Ok(())
}

/// 浏览器清理
pub fn cleanup() {
    // 清理各模块
    settings::cleanup();
    extensions::cleanup();
    ui::cleanup();
    javascript::cleanup();
    renderer::cleanup();
    storage::cleanup();
    network::cleanup();
    core::cleanup();
}

/// 浏览器
pub struct Browser {
    core: BrowserCore,
    renderer: HtmlRenderer,
    network: NetworkManager,
    storage: StorageManager,
    ui: BrowserUI,
    settings: BrowserSettings,
}

impl Browser {
    /// 创建浏览器
    pub fn new() -> Self {
        Self {
            core: BrowserCore::new(),
            renderer: HtmlRenderer::new(),
            network: NetworkManager::new(),
            storage: StorageManager::new(),
            ui: BrowserUI::new(),
            settings: BrowserSettings::default(),
        }
    }

    /// 加载网页
    pub fn load_url(&mut self, url: &str) -> Result<(), crate::error::BrowserError> {
        self.core.load_url(url)
    }

    /// 前进
    pub fn go_forward(&mut self) -> Result<(), crate::error::BrowserError> {
        self.core.go_forward()
    }

    /// 后退
    pub fn go_back(&mut self) -> Result<(), crate::error::BrowserError> {
        self.core.go_back()
    }

    /// 刷新
    pub fn refresh(&mut self) -> Result<(), crate::error::BrowserError> {
        self.core.refresh()
    }

    /// 停止加载
    pub fn stop(&mut self) -> Result<(), crate::error::BrowserError> {
        self.core.stop()
    }

    /// 搜索
    pub fn search(&mut self, query: &str) -> Result<(), crate::error::BrowserError> {
        self.core.search(query)
    }

    /// 获取设置
    pub fn settings(&self) -> &BrowserSettings {
        &self.settings
    }

    /// 更新设置
    pub fn update_settings(&mut self, settings: BrowserSettings) {
        self.settings = settings;
    }
}

/// 错误模块
pub mod error {
    #[derive(Debug)]
    pub enum BrowserError {
        InitializationError,
        NetworkError,
        RenderingError,
        StorageError,
        JavaScriptError,
        UrlError,
        SecurityError,
    }

    impl From<core::fmt::Error> for BrowserError {
        fn from(_: core::fmt::Error) -> Self {
            BrowserError::RenderingError
        }
    }
}


