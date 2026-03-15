#![no_std]

// 网络服务

pub mod drivers;
pub mod link;
pub mod network;
pub mod transport;
pub mod utils;
pub mod config;

// 初始化网络服务
pub fn init() {
    // 初始化网络配置
    config::init();
    
    // 初始化网卡驱动
    drivers::init();
    
    // 初始化链路层
    link::init();
    
    // 初始化网络层
    network::init();
    
    // 初始化传输层
    transport::init();
}

// 网络服务错误类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkError {
    InvalidArgument,
    ResourceBusy,
    NotFound,
    PermissionDenied,
    NotSupported,
    InternalError,
    NetworkUnreachable,
    HostUnreachable,
    ConnectionRefused,
    Timeout,
}

