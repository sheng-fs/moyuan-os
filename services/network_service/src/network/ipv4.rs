// IPv4协议实现

// IPv4头部结构
#[repr(packed)]
#[derive(Clone)]
pub struct Ipv4Header {
    pub version: u8,          // 版本（4位）
    pub ihl: u8,              // 头部长度（4位）
    pub tos: u8,              // 服务类型
    pub total_length: u16,     // 总长度
    pub identification: u16,   // 标识
    pub flags: u8,             // 标志（3位）
    pub fragment_offset: u16,  // 片偏移
    pub ttl: u8,               // 生存时间
    pub protocol: u8,          // 协议
    pub checksum: u16,         // 校验和
    pub source_ip: [u8; 4],    // 源IP地址
    pub destination_ip: [u8; 4], // 目标IP地址
}

// 计算IPv4头部校验和
pub fn calculate_checksum(header: &Ipv4Header) -> u16 {
    let header_bytes = unsafe {
        core::slice::from_raw_parts(
            header as *const Ipv4Header as *const u8,
            core::mem::size_of::<Ipv4Header>()
        )
    };
    
    let mut sum: u32 = 0;
    let mut i = 0;
    
    // 处理偶数长度的数据
    while i + 1 < header_bytes.len() {
        let word = ((header_bytes[i] as u32) << 8) | (header_bytes[i + 1] as u32);
        sum += word;
        i += 2;
    }
    
    // 处理奇数长度的数据
    if i < header_bytes.len() {
        sum += (header_bytes[i] as u32) << 8;
    }
    
    // 折叠校验和
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    
    // 取反
    !sum as u16
}

// 解析IPv4数据包
pub fn parse_ipv4_packet(data: &[u8]) -> Result<Ipv4Header, crate::NetworkError> {
    if data.len() < core::mem::size_of::<Ipv4Header>() {
        return Err(crate::NetworkError::InvalidArgument);
    }
    
    let header = unsafe {
        &*(data.as_ptr() as *const Ipv4Header)
    };
    
    if header.version != 4 || header.ihl < 5 {
        return Err(crate::NetworkError::InvalidArgument);
    }
    
    Ok(header.clone())
}
