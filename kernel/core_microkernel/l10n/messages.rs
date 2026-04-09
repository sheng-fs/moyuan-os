//! 多语言消息管理

use super::*;

/// 消息ID枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageId {
    // 通用消息
    Welcome,
    Error,
    Warning,
    Info,
    Success,

    // 系统消息
    SystemBooting,
    SystemShuttingDown,
    SystemReady,

    // 网络消息
    NetworkConnected,
    NetworkDisconnected,
    NetworkError,

    // 文件系统消息
    FileNotFound,
    PermissionDenied,
    DiskFull,

    // 登录消息
    LoginPrompt,
    PasswordPrompt,
    LoginSuccess,
    LoginFailed,

    // 其他消息
    Yes,
    No,
    OK,
    Cancel,
    Save,
    Open,
    Close,
    Delete,
    Create,
    Edit,
    View,
    Help,
    About,
    Exit,
}

/// 消息包
pub struct MessageBundle {
    messages: std::collections::HashMap<MessageId, String>,
}

impl MessageBundle {
    /// 创建新的消息包
    pub fn new() -> Self {
        Self {
            messages: std::collections::HashMap::new(),
        }
    }

    /// 添加消息
    pub fn add_message(&mut self, id: MessageId, message: &str) {
        self.messages.insert(id, message.to_string());
    }

    /// 获取消息
    pub fn get_message(&self, id: MessageId) -> Option<&String> {
        self.messages.get(&id)
    }
}

/// 消息管理器
pub struct MessageManager {
    message_bundles: std::collections::HashMap<LanguageCode, MessageBundle>,
    current_language: LanguageCode,
}

impl MessageManager {
    /// 创建消息管理器
    pub fn new() -> Self {
        let mut manager = Self {
            message_bundles: std::collections::HashMap::new(),
            current_language: LanguageCode::Default,
        };

        // 初始化默认消息包
        manager.init_default_messages();

        manager
    }

    /// 初始化默认消息
    fn init_default_messages(&mut self) {
        // 简体中文消息
        let mut zh_cn_bundle = MessageBundle::new();
        zh_cn_bundle.add_message(MessageId::Welcome, "欢迎使用墨渊操作系统");
        zh_cn_bundle.add_message(MessageId::Error, "错误");
        zh_cn_bundle.add_message(MessageId::Warning, "警告");
        zh_cn_bundle.add_message(MessageId::Info, "信息");
        zh_cn_bundle.add_message(MessageId::Success, "成功");
        zh_cn_bundle.add_message(MessageId::SystemBooting, "系统正在启动...");
        zh_cn_bundle.add_message(MessageId::SystemShuttingDown, "系统正在关闭...");
        zh_cn_bundle.add_message(MessageId::SystemReady, "系统已就绪");
        zh_cn_bundle.add_message(MessageId::NetworkConnected, "网络已连接");
        zh_cn_bundle.add_message(MessageId::NetworkDisconnected, "网络已断开");
        zh_cn_bundle.add_message(MessageId::NetworkError, "网络错误");
        zh_cn_bundle.add_message(MessageId::FileNotFound, "文件未找到");
        zh_cn_bundle.add_message(MessageId::PermissionDenied, "权限不足");
        zh_cn_bundle.add_message(MessageId::DiskFull, "磁盘空间不足");
        zh_cn_bundle.add_message(MessageId::LoginPrompt, "请输入用户名：");
        zh_cn_bundle.add_message(MessageId::PasswordPrompt, "请输入密码：");
        zh_cn_bundle.add_message(MessageId::LoginSuccess, "登录成功");
        zh_cn_bundle.add_message(MessageId::LoginFailed, "登录失败");
        zh_cn_bundle.add_message(MessageId::Yes, "是");
        zh_cn_bundle.add_message(MessageId::No, "否");
        zh_cn_bundle.add_message(MessageId::OK, "确定");
        zh_cn_bundle.add_message(MessageId::Cancel, "取消");
        zh_cn_bundle.add_message(MessageId::Save, "保存");
        zh_cn_bundle.add_message(MessageId::Open, "打开");
        zh_cn_bundle.add_message(MessageId::Close, "关闭");
        zh_cn_bundle.add_message(MessageId::Delete, "删除");
        zh_cn_bundle.add_message(MessageId::Create, "创建");
        zh_cn_bundle.add_message(MessageId::Edit, "编辑");
        zh_cn_bundle.add_message(MessageId::View, "查看");
        zh_cn_bundle.add_message(MessageId::Help, "帮助");
        zh_cn_bundle.add_message(MessageId::About, "关于");
        zh_cn_bundle.add_message(MessageId::Exit, "退出");
        self.message_bundles.insert(LanguageCode::ChineseSimplified, zh_cn_bundle);

        // 英文消息
        let mut en_us_bundle = MessageBundle::new();
        en_us_bundle.add_message(MessageId::Welcome, "Welcome to Moyuan OS");
        en_us_bundle.add_message(MessageId::Error, "Error");
        en_us_bundle.add_message(MessageId::Warning, "Warning");
        en_us_bundle.add_message(MessageId::Info, "Information");
        en_us_bundle.add_message(MessageId::Success, "Success");
        en_us_bundle.add_message(MessageId::SystemBooting, "System booting...");
        en_us_bundle.add_message(MessageId::SystemShuttingDown, "System shutting down...");
        en_us_bundle.add_message(MessageId::SystemReady, "System ready");
        en_us_bundle.add_message(MessageId::NetworkConnected, "Network connected");
        en_us_bundle.add_message(MessageId::NetworkDisconnected, "Network disconnected");
        en_us_bundle.add_message(MessageId::NetworkError, "Network error");
        en_us_bundle.add_message(MessageId::FileNotFound, "File not found");
        en_us_bundle.add_message(MessageId::PermissionDenied, "Permission denied");
        en_us_bundle.add_message(MessageId::DiskFull, "Disk full");
        en_us_bundle.add_message(MessageId::LoginPrompt, "Username: ");
        en_us_bundle.add_message(MessageId::PasswordPrompt, "Password: ");
        en_us_bundle.add_message(MessageId::LoginSuccess, "Login successful");
        en_us_bundle.add_message(MessageId::LoginFailed, "Login failed");
        en_us_bundle.add_message(MessageId::Yes, "Yes");
        en_us_bundle.add_message(MessageId::No, "No");
        en_us_bundle.add_message(MessageId::OK, "OK");
        en_us_bundle.add_message(MessageId::Cancel, "Cancel");
        en_us_bundle.add_message(MessageId::Save, "Save");
        en_us_bundle.add_message(MessageId::Open, "Open");
        en_us_bundle.add_message(MessageId::Close, "Close");
        en_us_bundle.add_message(MessageId::Delete, "Delete");
        en_us_bundle.add_message(MessageId::Create, "Create");
        en_us_bundle.add_message(MessageId::Edit, "Edit");
        en_us_bundle.add_message(MessageId::View, "View");
        en_us_bundle.add_message(MessageId::Help, "Help");
        en_us_bundle.add_message(MessageId::About, "About");
        en_us_bundle.add_message(MessageId::Exit, "Exit");
        self.message_bundles.insert(LanguageCode::EnglishUS, en_us_bundle);
    }

    /// 设置当前语言
    pub fn set_language(&mut self, language: LanguageCode) {
        self.current_language = language;
    }

    /// 获取消息
    pub fn get_message(&self, id: MessageId) -> Option<&String> {
        if let Some(bundle) = self.message_bundles.get(&self.current_language) {
            if let Some(message) = bundle.get_message(id) {
                return Some(message);
            }
        }

        // 回退到默认语言
        if let Some(bundle) = self.message_bundles.get(&LanguageCode::Default) {
            bundle.get_message(id)
        } else {
            None
        }
    }

    /// 添加消息包
    pub fn add_message_bundle(&mut self, language: LanguageCode, bundle: MessageBundle) {
        self.message_bundles.insert(language, bundle);
    }

    /// 获取当前语言的消息包
    pub fn current_message_bundle(&self) -> Option<&MessageBundle> {
        self.message_bundles.get(&self.current_language)
    }
}

/// 全局消息管理器
static mut MESSAGE_MANAGER: Option<MessageManager> = None;

/// 消息模块初始化
pub fn init() -> Result<(), crate::error::KernelError> {
    crate::println!("Initializing messages module...");

    unsafe {
        MESSAGE_MANAGER = Some(MessageManager::new());
    }

    crate::println!("Messages module initialized successfully");
    Ok(())
}

/// 消息模块清理
pub fn cleanup() {
    // TODO: 实现清理逻辑
}

/// 获取消息管理器
pub fn get_message_manager() -> &'static mut MessageManager {
    unsafe {
        MESSAGE_MANAGER.as_mut().expect("Message manager not initialized")
    }
}

/// 获取消息
pub fn get_message(id: MessageId) -> Option<&'static str> {
    let manager = get_message_manager();
    manager.get_message(id).map(|s| s.as_str())
}

/// 设置消息语言
pub fn set_message_language(language: LanguageCode) {
    get_message_manager().set_language(language);
}
