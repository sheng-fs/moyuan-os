// ICMP协议实现

// ICMP类型
pub const ICMP_TYPE_ECHO_REPLY: u8 = 0;
pub const ICMP_TYPE_ECHO_REQUEST: u8 = 8;

// ICMP头部结构
#[repr(packed)]
pub struct IcmpHeader {
    pub icmp_type: u8,      // 类型
    pub code: u8,           // 代码
    pub checksum: u16,      // 校验和
    pub identifier: u16,     // 标识符
    pub sequence_number: u16, // 序列号
}

// ICMP回显请求/响应数据结构
pub struct IcmpEcho {
    pub header: IcmpHeader,
    pub data: &'static [u8],
}

// 处理ICMP数据包
pub fn handle_icmp_packet(data: &[u8]) -> Result<(), crate::NetworkError> {
    if data.len() < core::mem::size_of::<IcmpHeader>() {
        return Err(crate::NetworkError::InvalidArgument);
    }
    
    let header = unsafe {
        &*(data.as_ptr() as *const IcmpHeader)
    };
    
    match header.icmp_type {
        ICMP_TYPE_ECHO_REQUEST => {
            // 处理回显请求（ping请求）
            handle_echo_request(data)?;
        }
        ICMP_TYPE_ECHO_REPLY => {
            // 处理回显响应（ping响应）
            handle_echo_reply(data);
        }
        _ => {
            // 其他ICMP类型
            // println!("Unknown ICMP type: {}", header.icmp_type);
        }
    }
    
    Ok(())
}

// 处理回显请求
fn handle_echo_request(data: &[u8]) -> Result<(), crate::NetworkError> {
    let header = unsafe {
        &*(data.as_ptr() as *const IcmpHeader)
    };
    
    // 构建回显响应
    let mut response_header = IcmpHeader {
        icmp_type: ICMP_TYPE_ECHO_REPLY,
        code: 0,
        checksum: 0,
        identifier: header.identifier,
        sequence_number: header.sequence_number,
    };
    
    // 构建响应数据
    let mut response_data = [0u8; 1500];
    let header_size = core::mem::size_of::<IcmpHeader>();
    
    // 复制头部
    unsafe {
        core::ptr::copy(&response_header as *const IcmpHeader as *const u8, 
                       response_data.as_mut_ptr(), 
                       header_size);
        
        // 复制数据部分
        if data.len() > header_size {
            core::ptr::copy(data.as_ptr().add(header_size),
                           response_data.as_mut_ptr().add(header_size),
                           data.len() - header_size);
        }
    }
    
    // 计算校验和
    let response_length = header_size + (data.len() - header_size);
    response_header.checksum = calculate_checksum(&response_data[..response_length]);
    
    // 重新复制头部（包含校验和）
    unsafe {
        core::ptr::copy(&response_header as *const IcmpHeader as *const u8, 
                       response_data.as_mut_ptr(), 
                       header_size);
    }
    
    // 发送回显响应
    // 注意：这里需要从IPv4数据包中获取源IP地址
    // 简化处理，暂时使用本地IP地址
    let _local_ip = crate::network::get_local_ip();
    let dest_ip = [127, 0, 0, 1]; // 暂时使用环回地址
    
    crate::network::send_packet(dest_ip, 1, &response_data[..response_length])
}

// 处理回显响应
fn handle_echo_reply(data: &[u8]) {
    let _header = unsafe {
        &*(data.as_ptr() as *const IcmpHeader)
    };
    
    // println!("Received ICMP echo reply: id={}, seq={}", 
    //          _header.identifier, _header.sequence_number);
}

// 计算ICMP校验和
fn calculate_checksum(data: &[u8]) -> u16 {
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

// 发送ping请求
pub fn send_ping(dest_ip: [u8; 4]) -> Result<(), crate::NetworkError> {
    // 构建ICMP回显请求
    let header = IcmpHeader {
        icmp_type: ICMP_TYPE_ECHO_REQUEST,
        code: 0,
        checksum: 0,
        identifier: 12345, // 随机标识符
        sequence_number: 1, // 序列号
    };
    
    // 构建请求数据
    let mut request_data = [0u8; 64];
    let header_size = core::mem::size_of::<IcmpHeader>();
    
    // 复制头部
    unsafe {
        core::ptr::copy(&header as *const IcmpHeader as *const u8, 
                       request_data.as_mut_ptr(), 
                       header_size);
        
        // 填充数据部分
        for i in header_size..64 {
            request_data[i] = i as u8;
        }
    }
    
    // 计算校验和
    let request_length = 64;
    let checksum = calculate_checksum(&request_data[..request_length]);
    
    // 更新校验和
    unsafe {
        let checksum_ptr = request_data.as_mut_ptr().offset(2);
        core::ptr::write(checksum_ptr as *mut u16, checksum);
    }
    
    // 发送ping请求
    crate::network::send_packet(dest_ip, 1, &request_data[..request_length])
}
