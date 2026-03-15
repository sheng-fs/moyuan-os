// 网络驱动模块

extern crate alloc;

pub mod e1000;
pub mod virtio_net;

use alloc::boxed::Box;
use crate::NetworkError;

// 网卡驱动接口
pub trait NetworkDriver {
    // 初始化驱动
    fn init(&mut self) -> Result<(), NetworkError>;
    
    // 发送数据包
    fn send_packet(&mut self, data: &[u8]) -> Result<(), NetworkError>;
    
    // 接收数据包
    fn receive_packet(&mut self, buffer: &mut [u8]) -> Result<usize, NetworkError>;
    
    // 获取MAC地址
    fn get_mac_address(&self) -> [u8; 6];
    
    // 检查驱动是否就绪
    fn is_ready(&self) -> bool;
}

// 驱动类型
pub enum DriverType {
    E1000,
    VirtioNet,
    Unknown,
}

// 全局驱动实例
static mut CURRENT_DRIVER: Option<Box<dyn NetworkDriver>> = None;

// 初始化驱动
pub fn init() {
    // 尝试初始化e1000驱动
    if let Ok(mut e1000_driver) = e1000::E1000Driver::new() {
        if e1000_driver.init().is_ok() {
            unsafe {
                CURRENT_DRIVER = Some(Box::new(e1000_driver));
            }
            return;
        }
    }
    
    // 尝试初始化virtio-net驱动
    if let Ok(mut virtio_driver) = virtio_net::VirtioNetDriver::new() {
        if virtio_driver.init().is_ok() {
            unsafe {
                CURRENT_DRIVER = Some(Box::new(virtio_driver));
            }
            return;
        }
    }
    
    // 没有找到可用的网卡驱动
    panic!("No network driver found");
}

// 获取当前驱动
pub fn get_driver() -> Option<&'static mut Box<dyn NetworkDriver>> {
    unsafe {
        (*(&raw mut CURRENT_DRIVER)).as_mut()
    }
}

// 发送数据包
pub fn send_packet(data: &[u8]) -> Result<(), NetworkError> {
    if let Some(driver) = get_driver() {
        driver.send_packet(data)
    } else {
        Err(NetworkError::NotSupported)
    }
}

// 接收数据包
pub fn receive_packet(buffer: &mut [u8]) -> Result<usize, NetworkError> {
    if let Some(driver) = get_driver() {
        driver.receive_packet(buffer)
    } else {
        Err(NetworkError::NotSupported)
    }
}

// 获取MAC地址
pub fn get_mac_address() -> [u8; 6] {
    if let Some(driver) = get_driver() {
        driver.get_mac_address()
    } else {
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    }
}