//! ARP 协议实现

use super::*;

/// ARP 协议初始化
pub fn init() -> Result<(), crate::error::KernelError> {
    crate::println!("Initializing ARP protocol...");
    // TODO: 实现 ARP 协议初始化
    Ok(())
}

/// ARP 协议清理
pub fn cleanup() {
    // TODO: 实现 ARP 协议清理
}

/// ARP 操作码
pub enum ArpOperation {
    Request = 1,
    Reply = 2,
}

/// ARP 缓存项
pub struct ArpCacheEntry {
    pub ip_address: [u8; 4],
    pub mac_address: [u8; 6],
    pub timestamp: u64,
}

/// ARP 缓存
pub struct ArpCache {
    entries: Vec<ArpCacheEntry>,
}

impl ArpCache {
    /// 创建新的 ARP 缓存
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// 添加缓存项
    pub fn add_entry(&mut self, ip_address: [u8; 4], mac_address: [u8; 6]) {
        self.entries.push(ArpCacheEntry {
            ip_address,
            mac_address,
            timestamp: crate::time::get_timestamp(),
        });
    }

    /// 查找缓存项
    pub fn find_entry(&self, ip_address: [u8; 4]) -> Option<&ArpCacheEntry> {
        self.entries.iter().find(|entry| entry.ip_address == ip_address)
    }
}
