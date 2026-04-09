//! IPv4 协议实现

use super::*;

/// IPv4 协议初始化
pub fn init() -> Result<(), crate::error::KernelError> {
    crate::println!("Initializing IPv4 protocol...");
    // TODO: 实现 IPv4 协议初始化
    Ok(())
}

/// IPv4 协议清理
pub fn cleanup() {
    // TODO: 实现 IPv4 协议清理
}

/// IPv4 地址
pub struct Ipv4Address {
    pub octets: [u8; 4],
}

impl Ipv4Address {
    /// 创建新的 IPv4 地址
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self { octets: [a, b, c, d] }
    }

    /// 从字符串创建 IPv4 地址
    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return None;
        }

        let mut octets = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            octets[i] = part.parse().ok()?;
        }

        Some(Self { octets })
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}.{}", self.octets[0], self.octets[1], self.octets[2], self.octets[3])
    }
}

/// IPv4 数据包
pub struct Ipv4Packet {
    pub version: u8,
    pub header_length: u8,
    pub tos: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags: u8,
    pub fragment_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub source_address: Ipv4Address,
    pub destination_address: Ipv4Address,
    pub options: Vec<u8>,
    pub payload: Vec<u8>,
}
