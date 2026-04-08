// 网络工具模块

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use core::fmt::Write;

// 计算校验和
pub fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    
    // 处理偶数长度的数据
    while i + 1 < data.len() {
        let word = ((data[i] as u32) << 8) | (data[i + 1] as u32);
        sum += word;
        i += 2;
    }
    
    // 处理奇数长度的数据
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    
    // 折叠校验和
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    
    // 取反
    !sum as u16
}

// 十六进制转MAC地址
pub fn hex_to_mac(hex: &str) -> Option<[u8; 6]> {
    let parts: Vec<&str> = hex.split(':').collect();
    if parts.len() != 6 {
        return None;
    }
    
    let mut mac = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        match u8::from_str_radix(part, 16) {
            Ok(value) => mac[i] = value,
            Err(_) => return None,
        }
    }
    
    Some(mac)
}

// MAC地址转十六进制
pub fn mac_to_hex(mac: &[u8; 6]) -> String {
    let mut result = String::new();
    for (i, &byte) in mac.iter().enumerate() {
        if i > 0 {
            result.push(':');
        }
        write!(&mut result, "{:02x}", byte).unwrap();
    }
    result
}

// 十六进制转IP地址
pub fn hex_to_ip(hex: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = hex.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    
    let mut ip = [0u8; 4];
    for (i, part) in parts.iter().enumerate() {
        match part.parse::<u8>() {
            Ok(value) => ip[i] = value,
            Err(_) => return None,
        }
    }
    
    Some(ip)
}

// IP地址转十六进制
pub fn ip_to_hex(ip: &[u8; 4]) -> String {
    let mut result = String::new();
    for (i, &byte) in ip.iter().enumerate() {
        if i > 0 {
            result.push('.');
        }
        write!(&mut result, "{}", byte).unwrap();
    }
    result
}
