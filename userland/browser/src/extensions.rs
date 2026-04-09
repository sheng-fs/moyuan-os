//! 扩展管理

use alloc::string::String;
use alloc::vec::Vec;
use super::*;

/// 扩展
pub struct Extension {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
}

/// 扩展管理器
pub struct ExtensionManager {
    extensions: Vec<Extension>,
}

impl ExtensionManager {
    /// 创建扩展管理器
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    /// 添加扩展
    pub fn add_extension(&mut self, extension: Extension) {
        self.extensions.push(extension);
    }

    /// 获取扩展列表
    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }

    /// 启用扩展
    pub fn enable_extension(&mut self, name: &str) -> Result<(), error::BrowserError> {
        if let Some(extension) = self.extensions.iter_mut().find(|ext| ext.name == name) {
            extension.enabled = true;
            Ok(())
        } else {
            Err(error::BrowserError::InitializationError)
        }
    }

    /// 禁用扩展
    pub fn disable_extension(&mut self, name: &str) -> Result<(), error::BrowserError> {
        if let Some(extension) = self.extensions.iter_mut().find(|ext| ext.name == name) {
            extension.enabled = false;
            Ok(())
        } else {
            Err(error::BrowserError::InitializationError)
        }
    }

    /// 移除扩展
    pub fn remove_extension(&mut self, name: &str) -> Result<(), error::BrowserError> {
        if let Some(index) = self.extensions.iter().position(|ext| ext.name == name) {
            self.extensions.remove(index);
            Ok(())
        } else {
            Err(error::BrowserError::InitializationError)
        }
    }
}

/// 扩展模块初始化
pub fn init() -> Result<(), error::BrowserError> {
    // TODO: 实现扩展模块初始化逻辑
    Ok(())
}

/// 扩展模块清理
pub fn cleanup() {
    // TODO: 实现扩展模块清理逻辑
}
