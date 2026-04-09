//! 系统本地化模块
//!
//! 实现操作系统的本地化支持，包括多语言界面、时区、字符编码等

pub mod locale;
pub mod timezone;
pub mod charset;
pub mod messages;

/// 本地化子系统初始化
pub fn init() -> Result<(), crate::error::KernelError> {
    crate::println!("Initializing localization subsystem...");

    // 初始化各本地化模块
    locale::init()?;
    timezone::init()?;
    charset::init()?;
    messages::init()?;

    crate::println!("Localization subsystem initialized successfully");
    Ok(())
}

/// 本地化子系统清理
pub fn cleanup() {
    crate::println!("Cleaning up localization subsystem...");

    // 清理各本地化模块
    messages::cleanup();
    charset::cleanup();
    timezone::cleanup();
    locale::cleanup();

    crate::println!("Localization subsystem cleanup completed");
}
