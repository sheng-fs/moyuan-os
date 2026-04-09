//! 语言环境管理

use super::*;

/// 语言代码枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageCode {
    // 中文
    ChineseSimplified, // 简体中文
    ChineseTraditional, // 繁体中文

    // 英语
    EnglishUS, // 美国英语
    EnglishUK, // 英国英语

    // 其他语言
    Japanese,
    Korean,
    French,
    German,
    Spanish,
    Russian,
    Arabic,
    Hindi,
    Portuguese,

    // 默认语言
    Default = ChineseSimplified as isize,
}

impl LanguageCode {
    /// 获取语言代码字符串
    pub fn code(&self) -> &'static str {
        match self {
            LanguageCode::ChineseSimplified => "zh-CN",
            LanguageCode::ChineseTraditional => "zh-TW",
            LanguageCode::EnglishUS => "en-US",
            LanguageCode::EnglishUK => "en-GB",
            LanguageCode::Japanese => "ja-JP",
            LanguageCode::Korean => "ko-KR",
            LanguageCode::French => "fr-FR",
            LanguageCode::German => "de-DE",
            LanguageCode::Spanish => "es-ES",
            LanguageCode::Russian => "ru-RU",
            LanguageCode::Arabic => "ar-SA",
            LanguageCode::Hindi => "hi-IN",
            LanguageCode::Portuguese => "pt-PT",
        }
    }

    /// 获取语言名称
    pub fn name(&self) -> &'static str {
        match self {
            LanguageCode::ChineseSimplified => "简体中文",
            LanguageCode::ChineseTraditional => "繁體中文",
            LanguageCode::EnglishUS => "English (US)",
            LanguageCode::EnglishUK => "English (UK)",
            LanguageCode::Japanese => "日本語",
            LanguageCode::Korean => "한국어",
            LanguageCode::French => "Français",
            LanguageCode::German => "Deutsch",
            LanguageCode::Spanish => "Español",
            LanguageCode::Russian => "Русский",
            LanguageCode::Arabic => "العربية",
            LanguageCode::Hindi => "हिन्दी",
            LanguageCode::Portuguese => "Português",
        }
    }
}

/// 语言环境配置
pub struct LocaleConfig {
    pub language: LanguageCode,
    pub region: String,
    pub charset: String,
    pub timezone: String,
}

impl LocaleConfig {
    /// 创建默认语言环境配置
    pub fn default() -> Self {
        Self {
            language: LanguageCode::Default,
            region: "CN".to_string(),
            charset: "UTF-8".to_string(),
            timezone: "Asia/Shanghai".to_string(),
        }
    }

    /// 创建自定义语言环境配置
    pub fn new(language: LanguageCode, region: &str, charset: &str, timezone: &str) -> Self {
        Self {
            language,
            region: region.to_string(),
            charset: charset.to_string(),
            timezone: timezone.to_string(),
        }
    }
}

/// 语言环境管理器
pub struct LocaleManager {
    current_locale: LocaleConfig,
    available_locales: Vec<LanguageCode>,
}

impl LocaleManager {
    /// 创建语言环境管理器
    pub fn new() -> Self {
        Self {
            current_locale: LocaleConfig::default(),
            available_locales: vec![
                LanguageCode::ChineseSimplified,
                LanguageCode::ChineseTraditional,
                LanguageCode::EnglishUS,
                LanguageCode::EnglishUK,
                LanguageCode::Japanese,
                LanguageCode::Korean,
                LanguageCode::French,
                LanguageCode::German,
                LanguageCode::Spanish,
                LanguageCode::Russian,
                LanguageCode::Arabic,
                LanguageCode::Hindi,
                LanguageCode::Portuguese,
            ],
        }
    }

    /// 获取当前语言环境
    pub fn current_locale(&self) -> &LocaleConfig {
        &self.current_locale
    }

    /// 设置语言环境
    pub fn set_locale(&mut self, locale: LocaleConfig) {
        self.current_locale = locale;
    }

    /// 获取可用的语言列表
    pub fn available_languages(&self) -> &[LanguageCode] {
        &self.available_locales
    }

    /// 检查语言是否可用
    pub fn is_language_available(&self, language: LanguageCode) -> bool {
        self.available_locales.contains(&language)
    }
}

/// 全局语言环境管理器
static mut LOCALE_MANAGER: Option<LocaleManager> = None;

/// 语言环境模块初始化
pub fn init() -> Result<(), crate::error::KernelError> {
    crate::println!("Initializing locale module...");

    unsafe {
        LOCALE_MANAGER = Some(LocaleManager::new());
    }

    crate::println!("Locale module initialized successfully");
    Ok(())
}

/// 语言环境模块清理
pub fn cleanup() {
    // TODO: 实现清理逻辑
}

/// 获取语言环境管理器
pub fn get_locale_manager() -> &'static mut LocaleManager {
    unsafe {
        LOCALE_MANAGER.as_mut().expect("Locale manager not initialized")
    }
}

/// 设置当前语言
pub fn set_language(language: LanguageCode) {
    let mut manager = get_locale_manager();
    let mut config = manager.current_locale().clone();
    config.language = language;
    manager.set_locale(config);
}

/// 获取当前语言
pub fn get_current_language() -> LanguageCode {
    get_locale_manager().current_locale().language
}
