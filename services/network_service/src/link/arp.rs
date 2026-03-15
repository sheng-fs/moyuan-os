// ARP协议模块

use crate::NetworkError;
use super::EthernetHeader;

// ARP操作码
const ARP_OP_REQUEST: u16 = 1;
const ARP_OP_REPLY: u16 = 2;

// ARP数据包
#[repr(C, packed)]
pub struct ArpPacket {
    pub hardware_type: u16,
    pub protocol_type: u16,
    pub hardware_length: u8,
    pub protocol_length: u8,
    pub operation: u16,
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}

// ARP缓存项
struct ArpCacheEntry {
    ip_address: [u8; 4],
    mac_address: [u8; 6],
    timestamp: u64,
    valid: bool,
}

// ARP缓存
const ARP_CACHE_SIZE: usize = 32;

// 默认ARP缓存项
const DEFAULT_ARP_CACHE_ENTRY: ArpCacheEntry = ArpCacheEntry {
    ip_address: [0; 4],
    mac_address: [0; 6],
    timestamp: 0,
    valid: false,
};

// ARP缓存
static mut ARP_CACHE: [ArpCacheEntry; ARP_CACHE_SIZE] = [DEFAULT_ARP_CACHE_ENTRY; ARP_CACHE_SIZE];

// 初始化ARP模块
pub fn init() {
    // 初始化ARP缓存
    unsafe {
        let cache = &mut *(&raw mut ARP_CACHE);
        for entry in cache {
            entry.valid = false;
        }
    }
}

// 发送ARP请求
pub fn send_arp_request(target_ip: [u8; 4]) -> Result<(), NetworkError> {
    let local_mac = super::get_local_mac();
    let local_ip = crate::config::get_config().ip_address;
    
    // 构建ARP请求
    let arp_packet = ArpPacket {
        hardware_type: 1, // 以太网
        protocol_type: 0x0800, // IPv4
        hardware_length: 6, // MAC地址长度
        protocol_length: 4, // IP地址长度
        operation: ARP_OP_REQUEST.to_be(),
        sender_mac: local_mac,
        sender_ip: local_ip,
        target_mac: [0xff; 6], // 广播MAC
        target_ip,
    };
    
    // 序列化ARP数据包
    let mut buffer = [0u8; core::mem::size_of::<ArpPacket>()];
    unsafe {
        core::ptr::copy(&arp_packet as *const ArpPacket as *const u8, buffer.as_mut_ptr(), core::mem::size_of::<ArpPacket>());
    }
    
    // 发送ARP请求
    super::send_frame([0xff; 6], 0x0806, &buffer)
}

// 处理ARP数据包
pub fn handle_arp_packet(data: &[u8]) -> Result<(), NetworkError> {
    if data.len() < core::mem::size_of::<ArpPacket>() {
        return Err(NetworkError::InvalidArgument);
    }
    
    let arp_packet = unsafe {
        &*(data.as_ptr() as *const ArpPacket)
    };
    
    let operation = u16::from_be(arp_packet.operation);
    
    match operation {
        ARP_OP_REQUEST => {
            // 处理ARP请求
            handle_arp_request(arp_packet)
        },
        ARP_OP_REPLY => {
            // 处理ARP回复
            handle_arp_reply(arp_packet)
        },
        _ => {
            Err(NetworkError::NotSupported)
        }
    }
}

// 处理ARP请求
fn handle_arp_request(packet: &ArpPacket) -> Result<(), NetworkError> {
    let local_ip = crate::config::get_config().ip_address;
    
    // 检查是否是针对本地IP的请求
    if packet.target_ip != local_ip {
        return Ok(());
    }
    
    // 发送ARP回复
    let local_mac = super::get_local_mac();
    
    let arp_reply = ArpPacket {
        hardware_type: 1, // 以太网
        protocol_type: 0x0800, // IPv4
        hardware_length: 6, // MAC地址长度
        protocol_length: 4, // IP地址长度
        operation: ARP_OP_REPLY.to_be(),
        sender_mac: local_mac,
        sender_ip: local_ip,
        target_mac: packet.sender_mac,
        target_ip: packet.sender_ip,
    };
    
    // 序列化ARP回复
    let mut buffer = [0u8; core::mem::size_of::<ArpPacket>()];
    unsafe {
        core::ptr::copy(&arp_reply as *const ArpPacket as *const u8, buffer.as_mut_ptr(), core::mem::size_of::<ArpPacket>());
    }
    
    // 发送ARP回复
    super::send_frame(packet.sender_mac, 0x0806, &buffer)
}

// 处理ARP回复
fn handle_arp_reply(packet: &ArpPacket) -> Result<(), NetworkError> {
    // 更新ARP缓存
    update_arp_cache(packet.sender_ip, packet.sender_mac);
    Ok(())
}

// 更新ARP缓存
fn update_arp_cache(ip: [u8; 4], mac: [u8; 6]) {
    unsafe {
        // 查找空闲或过期的缓存项
        let mut index = ARP_CACHE_SIZE;
        let cache = &*(&raw const ARP_CACHE);
        for (i, entry) in cache.iter().enumerate() {
            if !entry.valid || is_arp_cache_expired(entry.timestamp) {
                index = i;
                break;
            }
        }
        
        // 如果没有空闲项，使用最早的项
        if index == ARP_CACHE_SIZE {
            let mut oldest_time = u64::MAX;
            let cache = &*(&raw const ARP_CACHE);
            for (i, entry) in cache.iter().enumerate() {
                if entry.timestamp < oldest_time {
                    oldest_time = entry.timestamp;
                    index = i;
                }
            }
        }
        
        // 更新缓存项
        let cache = &mut *(&raw mut ARP_CACHE);
        cache[index].ip_address = ip;
        cache[index].mac_address = mac;
        cache[index].timestamp = get_current_timestamp();
        cache[index].valid = true;
    }
}

// 从ARP缓存中查找MAC地址
pub fn lookup_arp_cache(ip: [u8; 4]) -> Option<[u8; 6]> {
    unsafe {
        let cache = &*(&raw const ARP_CACHE);
        for entry in cache {
            if entry.valid && !is_arp_cache_expired(entry.timestamp) && entry.ip_address == ip {
                return Some(entry.mac_address);
            }
        }
    }
    None
}

// 检查ARP缓存是否过期
fn is_arp_cache_expired(timestamp: u64) -> bool {
    let current_time = get_current_timestamp();
    let timeout = crate::config::get_config().arp_cache_timeout as u64;
    current_time - timestamp > timeout
}

// 获取当前时间戳（简化实现）
fn get_current_timestamp() -> u64 {
    // 这里使用一个简单的计数器作为时间戳
    static mut TIMESTAMP: u64 = 0;
    unsafe {
        TIMESTAMP += 1;
        TIMESTAMP
    }
}

// 解析IP地址为MAC地址
pub fn resolve_ip_to_mac(ip: [u8; 4]) -> Result<[u8; 6], NetworkError> {
    // 首先从缓存中查找
    if let Some(mac) = lookup_arp_cache(ip) {
        return Ok(mac);
    }
    
    // 发送ARP请求
    send_arp_request(ip)?;
    
    // 等待ARP回复（这里简化处理，实际应该使用超时机制）
    for _ in 0..10 {
        let mut buffer = [0u8; 1514];
        match super::receive_frame(&mut buffer) {
            Ok((header, length)) => {
                if header.ethertype == 0x0806 && length >= core::mem::size_of::<ArpPacket>() {
                    let arp_packet = unsafe {
                        &*(buffer[core::mem::size_of::<EthernetHeader>()..].as_ptr() as *const ArpPacket)
                    };
                    if u16::from_be(arp_packet.operation) == ARP_OP_REPLY && arp_packet.sender_ip == ip {
                        return Ok(arp_packet.sender_mac);
                    }
                }
            },
            Err(_) => {
                // 忽略错误，继续等待
            }
        }
    }
    
    Err(NetworkError::Timeout)
}