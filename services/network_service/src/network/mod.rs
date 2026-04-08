// 网络层模块

pub mod ipv4;
pub mod icmp;

use crate::NetworkError;

// 网络层接口
pub trait NetworkLayer {
    // 发送数据包
    fn send_packet(&mut self, dest_ip: [u8; 4], protocol: u8, data: &[u8]) -> Result<(), NetworkError>;
    
    // 接收数据包
    fn receive_packet(&mut self, buffer: &mut [u8], length: usize) -> Result<(), NetworkError>;
    
    // 获取本地IP地址
    fn get_local_ip(&self) -> [u8; 4];
}

// IPv4网络层实现
pub struct Ipv4NetworkLayer {
    local_ip: [u8; 4],
    subnet_mask: [u8; 4],
    default_gateway: [u8; 4],
}

impl Ipv4NetworkLayer {
    // 创建新的IPv4网络层实例
    pub fn new() -> Self {
        let config = crate::config::get_config();
        Self {
            local_ip: config.ip_address,
            subnet_mask: config.subnet_mask,
            default_gateway: config.default_gateway,
        }
    }
    
    // 检查IP地址是否在本地子网
    fn is_in_local_subnet(&self, ip: [u8; 4]) -> bool {
        for i in 0..4 {
            if (ip[i] & self.subnet_mask[i]) != (self.local_ip[i] & self.subnet_mask[i]) {
                return false;
            }
        }
        true
    }
}

impl NetworkLayer for Ipv4NetworkLayer {
    // 发送数据包
    fn send_packet(&mut self, dest_ip: [u8; 4], protocol: u8, data: &[u8]) -> Result<(), NetworkError> {
        // 确定下一跳IP地址
        let next_hop_ip = if self.is_in_local_subnet(dest_ip) {
            dest_ip
        } else {
            self.default_gateway
        };
        
        // 解析下一跳IP地址为MAC地址
        let dest_mac = crate::link::arp::resolve_ip_to_mac(next_hop_ip)?;
        
        // 构建IPv4头部
        let mut ip_header = ipv4::Ipv4Header {
            version: 4,
            ihl: 5,
            tos: 0,
            total_length: (core::mem::size_of::<ipv4::Ipv4Header>() + data.len()) as u16,
            identification: 0,
            flags: 0,
            fragment_offset: 0,
            ttl: 64,
            protocol,
            checksum: 0,
            source_ip: self.local_ip,
            destination_ip: dest_ip,
        };
        
        // 计算校验和
        ip_header.checksum = ipv4::calculate_checksum(&ip_header);
        
        // 序列化IPv4头部
        let mut buffer = [0u8; 1500];
        let header_size = core::mem::size_of::<ipv4::Ipv4Header>();
        unsafe {
            core::ptr::copy(&ip_header as *const ipv4::Ipv4Header as *const u8, buffer.as_mut_ptr(), header_size);
            core::ptr::copy(data.as_ptr(), buffer.as_mut_ptr().add(header_size), data.len());
        }
        
        // 发送数据包
        crate::link::send_frame(dest_mac, crate::link::ETHERTYPE_IPV4, &buffer[..header_size + data.len()])
    }
    
    // 接收数据包
    fn receive_packet(&mut self, buffer: &mut [u8], length: usize) -> Result<(), NetworkError> {
        if length < core::mem::size_of::<ipv4::Ipv4Header>() {
            return Err(NetworkError::InvalidArgument);
        }
        
        // 解析IPv4头部
        let ip_header = unsafe {
            &*(buffer.as_ptr() as *const ipv4::Ipv4Header)
        };
        
        // 检查版本和头部长度
        if ip_header.version != 4 || ip_header.ihl < 5 {
            return Err(NetworkError::InvalidArgument);
        }
        
        // 检查目标IP地址
        if ip_header.destination_ip != self.local_ip {
            // 不是发给本地的数据包，忽略
            return Ok(());
        }
        
        // 计算校验和
        let checksum = ipv4::calculate_checksum(ip_header);
        if checksum != 0 {
            return Err(NetworkError::InvalidArgument);
        }
        
        // 处理不同协议的数据包
        let header_size = ip_header.ihl as usize * 4;
        let payload_size = length - header_size;
        
        match ip_header.protocol {
            1 => {
                // ICMP协议
                icmp::handle_icmp_packet(&buffer[header_size..header_size + payload_size])
            },
            6 => {
                // TCP协议
                Ok(())
            },
            17 => {
                // UDP协议
                crate::transport::udp::handle_udp_packet(&buffer[header_size..header_size + payload_size])
            },
            _ => {
                Ok(())
            }
        }
    }
    
    // 获取本地IP地址
    fn get_local_ip(&self) -> [u8; 4] {
        self.local_ip
    }
}

// 全局网络层实例
static mut NETWORK_LAYER: Option<Ipv4NetworkLayer> = None;

// 初始化网络层
pub fn init() {
    unsafe {
        NETWORK_LAYER = Some(Ipv4NetworkLayer::new());
    }
}

// 获取网络层实例
pub fn get_network_layer() -> Option<&'static mut Ipv4NetworkLayer> {
    unsafe {
        (*(&raw mut NETWORK_LAYER)).as_mut()
    }
}

// 发送数据包
pub fn send_packet(dest_ip: [u8; 4], protocol: u8, data: &[u8]) -> Result<(), NetworkError> {
    if let Some(network_layer) = get_network_layer() {
        network_layer.send_packet(dest_ip, protocol, data)
    } else {
        Err(NetworkError::NotSupported)
    }
}

// 接收数据包
pub fn receive_packet(buffer: &mut [u8], length: usize) -> Result<(), NetworkError> {
    if let Some(network_layer) = get_network_layer() {
        network_layer.receive_packet(buffer, length)
    } else {
        Err(NetworkError::NotSupported)
    }
}

// 获取本地IP地址
pub fn get_local_ip() -> [u8; 4] {
    if let Some(network_layer) = get_network_layer() {
        network_layer.get_local_ip()
    } else {
        [0, 0, 0, 0]
    }
}