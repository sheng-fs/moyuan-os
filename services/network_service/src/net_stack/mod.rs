// 网络栈模块

use crate::drivers;
use crate::link;
use crate::network;
use crate::transport;

// 初始化网络栈
pub fn init() {
    // 初始化链路层
    link::init();
    
    // 初始化网络层
    network::init();
    
    // 初始化传输层
    transport::init();
    
    // 初始化网络驱动
    drivers::init();
}

// 发送网络数据
pub fn send_data(
    dest_ip: [u8; 4],
    dest_port: u16,
    data: &[u8],
) -> Result<(), crate::NetworkError> {
    // 获取本地IP地址
    let source_ip = network::get_local_ip();
    
    // 随机源端口
    let source_port = 1024 + (unsafe { core::arch::x86_64::_rdtsc() } % 49152) as u16;
    
    // 通过传输层发送
    transport::udp::send_udp(source_ip, dest_ip, source_port, dest_port, data)
}

// 处理接收到的网络数据
pub fn handle_received_data(data: &mut [u8]) {
    // 解析以太网帧
    if let Ok((header, length)) = link::receive_frame(data) {
        // 处理不同类型的以太网帧
        match header.ethertype {
            link::ETHERTYPE_ARP => {
                // 处理ARP请求
                let _ = link::arp::handle_arp_packet(&data[core::mem::size_of::<link::EthernetHeader>()..]);
            }
            link::ETHERTYPE_IPV4 => {
                // 处理IPv4数据包
                if let Ok(packet) = network::ipv4::parse_ipv4_packet(&data[core::mem::size_of::<link::EthernetHeader>()..core::mem::size_of::<link::EthernetHeader>() + length]) {
                    // 处理不同类型的IPv4数据包
                    match packet.protocol {
                        1 => {
                            // ICMP协议
                            let _ = network::icmp::handle_icmp_packet(&data[core::mem::size_of::<link::EthernetHeader>() + (packet.ihl as usize * 4)..]);
                        }
                        17 => {
                            // UDP协议
                            // 简化处理，实际需要解析UDP头部
                            let _ = transport::udp::handle_udp_packet(&data[core::mem::size_of::<link::EthernetHeader>() + (packet.ihl as usize * 4)..]);
                        }
                        _ => {
                            // 其他协议
                            // 暂时注释掉println!，因为在no_std环境中不可用
                            // println!("Unknown protocol: {}", packet.protocol);
                        }
                    }
                }
            }
            _ => {
                // 其他以太网类型
                // 暂时注释掉println!，因为在no_std环境中不可用
                // println!("Unknown ethertype: {:x}", header.ethertype);
            }
        }
    }
}
