//! Shell 多国语言支持
//!
//! 为 Shell 提供多语言界面支持

#![no_std]
#![allow(unexpected_cfgs)]

pub mod messages;
pub mod translations;

use crate::messages::ShellMessageId;
use crate::translations::ShellTranslations;

/// Shell 本地化初始化
pub fn init() -> Result<(), crate::error::LocalizationError> {
    println!("Initializing shell localization...");

    // 初始化各模块
    messages::init()?;
    translations::init()?;

    println!("Shell localization initialized successfully");
    Ok(())
}

/// Shell 本地化清理
pub fn cleanup() {
    println!("Cleaning up shell localization...");

    // 清理各模块
    translations::cleanup();
    messages::cleanup();

    println!("Shell localization cleanup completed");
}

/// 获取本地化消息
pub fn get_message(id: ShellMessageId) -> &'static str {
    ShellTranslations::get_message(id)
}

/// 设置当前语言
pub fn set_language(language: &str) {
    ShellTranslations::set_language(language);
}

/// 获取当前语言
pub fn get_current_language() -> &'static str {
    ShellTranslations::current_language()
}

/// 错误模块
pub mod error {
    #[derive(Debug)]
    pub enum LocalizationError {
        InitializationError,
        TranslationError,
        LanguageNotFound,
    }

    impl From<core::fmt::Error> for LocalizationError {
        fn from(_: core::fmt::Error) -> Self {
            LocalizationError::TranslationError
        }
    }
}
