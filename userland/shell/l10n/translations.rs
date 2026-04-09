//! Shell 翻译管理

use super::*;
use crate::messages::ShellMessageId;

/// 翻译映射
type TranslationMap = std::collections::HashMap<ShellMessageId, &'static str>;

/// Shell 翻译
pub struct ShellTranslations {
    current_language: &'static str,
    translations: std::collections::HashMap<&'static str, TranslationMap>,
}

impl ShellTranslations {
    /// 获取全局翻译实例
    pub fn instance() -> &'static mut Self {
        static mut INSTANCE: Option<ShellTranslations> = None;
        unsafe {
            if INSTANCE.is_none() {
                INSTANCE = Some(Self::new());
            }
            INSTANCE.as_mut().unwrap()
        }
    }

    /// 创建 Shell 翻译
    fn new() -> Self {
        let mut translations = std::collections::HashMap::new();

        // 简体中文翻译
        let mut zh_cn = TranslationMap::new();
        zh_cn.insert(ShellMessageId::Welcome, "欢迎使用墨渊操作系统 Shell");
        zh_cn.insert(ShellMessageId::Prompt, "$ ");
        zh_cn.insert(ShellMessageId::Error, "错误");
        zh_cn.insert(ShellMessageId::Warning, "警告");
        zh_cn.insert(ShellMessageId::Info, "信息");
        zh_cn.insert(ShellMessageId::Success, "成功");
        zh_cn.insert(ShellMessageId::CommandNotFound, "命令未找到");
        zh_cn.insert(ShellMessageId::PermissionDenied, "权限不足");
        zh_cn.insert(ShellMessageId::InvalidSyntax, "语法错误");
        zh_cn.insert(ShellMessageId::TooManyArguments, "参数过多");
        zh_cn.insert(ShellMessageId::TooFewArguments, "参数不足");
        zh_cn.insert(ShellMessageId::FileNotFound, "文件未找到");
        zh_cn.insert(ShellMessageId::DirectoryNotFound, "目录未找到");
        zh_cn.insert(ShellMessageId::FileExists, "文件已存在");
        zh_cn.insert(ShellMessageId::DirectoryExists, "目录已存在");
        zh_cn.insert(ShellMessageId::SystemBooting, "系统正在启动...");
        zh_cn.insert(ShellMessageId::SystemShuttingDown, "系统正在关闭...");
        zh_cn.insert(ShellMessageId::SystemReady, "系统已就绪");
        zh_cn.insert(ShellMessageId::NetworkConnected, "网络已连接");
        zh_cn.insert(ShellMessageId::NetworkDisconnected, "网络已断开");
        zh_cn.insert(ShellMessageId::NetworkError, "网络错误");
        zh_cn.insert(ShellMessageId::LoginPrompt, "请输入用户名：");
        zh_cn.insert(ShellMessageId::PasswordPrompt, "请输入密码：");
        zh_cn.insert(ShellMessageId::LoginSuccess, "登录成功");
        zh_cn.insert(ShellMessageId::LoginFailed, "登录失败");
        zh_cn.insert(ShellMessageId::Yes, "是");
        zh_cn.insert(ShellMessageId::No, "否");
        zh_cn.insert(ShellMessageId::OK, "确定");
        zh_cn.insert(ShellMessageId::Cancel, "取消");
        zh_cn.insert(ShellMessageId::Help, "帮助");
        zh_cn.insert(ShellMessageId::Exit, "退出");
        zh_cn.insert(ShellMessageId::Clear, "清除");
        zh_cn.insert(ShellMessageId::History, "历史");
        zh_cn.insert(ShellMessageId::Jobs, "作业");
        zh_cn.insert(ShellMessageId::Foreground, "前台");
        zh_cn.insert(ShellMessageId::Background, "后台");
        translations.insert("zh-CN", zh_cn);

        // 英文翻译
        let mut en_us = TranslationMap::new();
        en_us.insert(ShellMessageId::Welcome, "Welcome to Moyuan OS Shell");
        en_us.insert(ShellMessageId::Prompt, "$ ");
        en_us.insert(ShellMessageId::Error, "Error");
        en_us.insert(ShellMessageId::Warning, "Warning");
        en_us.insert(ShellMessageId::Info, "Information");
        en_us.insert(ShellMessageId::Success, "Success");
        en_us.insert(ShellMessageId::CommandNotFound, "Command not found");
        en_us.insert(ShellMessageId::PermissionDenied, "Permission denied");
        en_us.insert(ShellMessageId::InvalidSyntax, "Invalid syntax");
        en_us.insert(ShellMessageId::TooManyArguments, "Too many arguments");
        en_us.insert(ShellMessageId::TooFewArguments, "Too few arguments");
        en_us.insert(ShellMessageId::FileNotFound, "File not found");
        en_us.insert(ShellMessageId::DirectoryNotFound, "Directory not found");
        en_us.insert(ShellMessageId::FileExists, "File exists");
        en_us.insert(ShellMessageId::DirectoryExists, "Directory exists");
        en_us.insert(ShellMessageId::SystemBooting, "System booting...");
        en_us.insert(ShellMessageId::SystemShuttingDown, "System shutting down...");
        en_us.insert(ShellMessageId::SystemReady, "System ready");
        en_us.insert(ShellMessageId::NetworkConnected, "Network connected");
        en_us.insert(ShellMessageId::NetworkDisconnected, "Network disconnected");
        en_us.insert(ShellMessageId::NetworkError, "Network error");
        en_us.insert(ShellMessageId::LoginPrompt, "Username: ");
        en_us.insert(ShellMessageId::PasswordPrompt, "Password: ");
        en_us.insert(ShellMessageId::LoginSuccess, "Login successful");
        en_us.insert(ShellMessageId::LoginFailed, "Login failed");
        en_us.insert(ShellMessageId::Yes, "Yes");
        en_us.insert(ShellMessageId::No, "No");
        en_us.insert(ShellMessageId::OK, "OK");
        en_us.insert(ShellMessageId::Cancel, "Cancel");
        en_us.insert(ShellMessageId::Help, "Help");
        en_us.insert(ShellMessageId::Exit, "Exit");
        en_us.insert(ShellMessageId::Clear, "Clear");
        en_us.insert(ShellMessageId::History, "History");
        en_us.insert(ShellMessageId::Jobs, "Jobs");
        en_us.insert(ShellMessageId::Foreground, "Foreground");
        en_us.insert(ShellMessageId::Background, "Background");
        translations.insert("en-US", en_us);

        Self {
            current_language: "zh-CN",
            translations,
        }
    }

    /// 获取消息
    pub fn get_message(id: ShellMessageId) -> &'static str {
        let instance = Self::instance();
        if let Some(translations) = instance.translations.get(instance.current_language) {
            if let Some(message) = translations.get(&id) {
                return message;
            }
        }

        // 回退到默认语言
        if let Some(translations) = instance.translations.get("zh-CN") {
            if let Some(message) = translations.get(&id) {
                return message;
            }
        }

        "Unknown message"
    }

    /// 设置语言
    pub fn set_language(language: &str) {
        let instance = Self::instance();
        if instance.translations.contains_key(language) {
            instance.current_language = language;
        }
    }

    /// 获取当前语言
    pub fn current_language() -> &'static str {
        Self::instance().current_language
    }

    /// 获取可用的语言列表
    pub fn available_languages() -> Vec<&'static str> {
        Self::instance().translations.keys().copied().collect()
    }
}

/// 翻译模块初始化
pub fn init() -> Result<(), error::LocalizationError> {
    println!("Initializing shell translations...");
    // 初始化翻译实例
    ShellTranslations::instance();
    Ok(())
}

/// 翻译模块清理
pub fn cleanup() {
    // TODO: 实现翻译模块清理逻辑
}
