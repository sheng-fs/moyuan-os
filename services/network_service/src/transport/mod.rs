// 传输层模块

pub mod udp;

// 传输层协议类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransportProtocol {
    Udp,
    Tcp,
}

// 传输层端口类型
#[allow(dead_code)]
type Port = u16;

// 初始化传输层
pub fn init() {
    udp::init();
}
