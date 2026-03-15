// 网络配置模块

use core::sync::atomic::{AtomicBool, Ordering};

// 网络配置结构
pub struct NetworkConfig {
    // IP地址
    pub ip_address: [u8; 4],
    // 子网掩码
    pub subnet_mask: [u8; 4],
    // 默认网关
    pub default_gateway: [u8; 4],
    // MAC地址
    pub mac_address: [u8; 6],
    // MTU大小
    pub mtu: u16,
    // ARP缓存超时时间（秒）
    pub arp_cache_timeout: u32,
}

// 全局网络配置
static mut NETWORK_CONFIG: NetworkConfig = NetworkConfig {
    ip_address: [192, 168, 1, 100],
    subnet_mask: [255, 255, 255, 0],
    default_gateway: [192, 168, 1, 1],
    mac_address: [0x52, 0x54, 0x00, 0x12, 0x34, 0x56],
    mtu: 1500,
    arp_cache_timeout: 300,
};

// 网络服务状态
static NETWORK_SERVICE_RUNNING: AtomicBool = AtomicBool::new(false);

// 初始化网络配置
pub fn init() {
    // 这里可以从配置文件或其他来源加载配置
    // 目前使用默认配置
    NETWORK_SERVICE_RUNNING.store(true, Ordering::SeqCst);
}

// 获取网络配置
pub fn get_config() -> &'static NetworkConfig {
    unsafe {
        &*(&raw const NETWORK_CONFIG)
    }
}

// 设置IP地址
pub fn set_ip_address(ip: [u8; 4]) {
    unsafe {
        (*(&raw mut NETWORK_CONFIG)).ip_address = ip;
    }
}

// 设置子网掩码
pub fn set_subnet_mask(mask: [u8; 4]) {
    unsafe {
        (*(&raw mut NETWORK_CONFIG)).subnet_mask = mask;
    }
}

// 设置默认网关
pub fn set_default_gateway(gateway: [u8; 4]) {
    unsafe {
        (*(&raw mut NETWORK_CONFIG)).default_gateway = gateway;
    }
}

// 设置MAC地址
pub fn set_mac_address(mac: [u8; 6]) {
    unsafe {
        (*(&raw mut NETWORK_CONFIG)).mac_address = mac;
    }
}

// 检查网络服务是否运行
pub fn is_running() -> bool {
    NETWORK_SERVICE_RUNNING.load(Ordering::SeqCst)
}

// 停止网络服务
pub fn stop() {
    NETWORK_SERVICE_RUNNING.store(false, Ordering::SeqCst);
}