// 链路层模块

pub mod arp;

use crate::NetworkError;

// 以太网帧头部
#[repr(C, packed)]
#[derive(Clone)]
pub struct EthernetHeader {
    pub dest_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: u16,
}

// 以太网类型
pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16 = 0x0806;

// 链路层接口
pub trait LinkLayer {
    // 发送数据帧
    fn send_frame(&mut self, dest_mac: [u8; 6], ethertype: u16, data: &[u8]) -> Result<(), NetworkError>;
    
    // 接收数据帧
    fn receive_frame(&mut self, buffer: &mut [u8]) -> Result<(EthernetHeader, usize), NetworkError>;
    
    // 获取本地MAC地址
    fn get_local_mac(&self) -> [u8; 6];
}

// 以太网链路层实现
pub struct EthernetLinkLayer {
    mac_address: [u8; 6],
}

impl EthernetLinkLayer {
    // 创建新的以太网链路层实例
    pub fn new() -> Self {
        let mac_address = crate::drivers::get_mac_address();
        Self {
            mac_address,
        }
    }
}

impl LinkLayer for EthernetLinkLayer {
    // 发送数据帧
    fn send_frame(&mut self, dest_mac: [u8; 6], ethertype: u16, data: &[u8]) -> Result<(), NetworkError> {
        // 计算帧长度
        let frame_length = core::mem::size_of::<EthernetHeader>() + data.len();
        if frame_length > 1514 {
            return Err(NetworkError::InvalidArgument);
        }
        
        // 构建以太网帧
        let mut frame = [0u8; 1514];
        let header = unsafe {
            &mut *(frame.as_mut_ptr() as *mut EthernetHeader)
        };
        
        header.dest_mac = dest_mac;
        header.src_mac = self.mac_address;
        header.ethertype = ethertype.to_be();
        
        // 复制数据
        unsafe {
            core::ptr::copy(data.as_ptr(), frame.as_mut_ptr().add(core::mem::size_of::<EthernetHeader>()), data.len());
        }
        
        // 发送帧
        crate::drivers::send_packet(&frame[..frame_length])
    }
    
    // 接收数据帧
    fn receive_frame(&mut self, buffer: &mut [u8]) -> Result<(EthernetHeader, usize), NetworkError> {
        // 接收数据包
        let length = crate::drivers::receive_packet(buffer)?;
        
        if length < core::mem::size_of::<EthernetHeader>() {
            return Err(NetworkError::InvalidArgument);
        }
        
        // 解析以太网头部
        let header = unsafe {
            &*(buffer.as_ptr() as *const EthernetHeader)
        };
        
        Ok((header.clone(), length - core::mem::size_of::<EthernetHeader>()))
    }
    
    // 获取本地MAC地址
    fn get_local_mac(&self) -> [u8; 6] {
        self.mac_address
    }
}

// 全局链路层实例
static mut LINK_LAYER: Option<EthernetLinkLayer> = None;

// 初始化链路层
pub fn init() {
    unsafe {
        LINK_LAYER = Some(EthernetLinkLayer::new());
    }
}

// 获取链路层实例
pub fn get_link_layer() -> Option<&'static mut EthernetLinkLayer> {
    unsafe {
        (*(&raw mut LINK_LAYER)).as_mut()
    }
}

// 发送数据帧
pub fn send_frame(dest_mac: [u8; 6], ethertype: u16, data: &[u8]) -> Result<(), NetworkError> {
    if let Some(link_layer) = get_link_layer() {
        link_layer.send_frame(dest_mac, ethertype, data)
    } else {
        Err(NetworkError::NotSupported)
    }
}

// 接收数据帧
pub fn receive_frame(buffer: &mut [u8]) -> Result<(EthernetHeader, usize), NetworkError> {
    if let Some(link_layer) = get_link_layer() {
        link_layer.receive_frame(buffer)
    } else {
        Err(NetworkError::NotSupported)
    }
}

// 获取本地MAC地址
pub fn get_local_mac() -> [u8; 6] {
    if let Some(link_layer) = get_link_layer() {
        link_layer.get_local_mac()
    } else {
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    }
}