//! 网络协议栈模块
//!
//! 实现内核级网络协议栈，支持多种网络协议

pub mod tcp;
pub mod udp;
pub mod icmp;
pub mod arp;
pub mod ipv4;
pub mod ipv6;
pub mod dns;
pub mod dhcp;

/// 网络协议栈初始化
pub fn init() -> Result<(), crate::error::KernelError> {
    crate::println!("Initializing network stack...");

    // 初始化各协议模块
    arp::init()?;
    ipv4::init()?;
    icmp::init()?;
    udp::init()?;
    tcp::init()?;

    crate::println!("Network stack initialized successfully");
    Ok(())
}

/// 网络协议栈清理
pub fn cleanup() {
    crate::println!("Cleaning up network stack...");

    // 清理各协议模块
    tcp::cleanup();
    udp::cleanup();
    icmp::cleanup();
    ipv4::cleanup();
    arp::cleanup();

    crate::println!("Network stack cleanup completed");
}
