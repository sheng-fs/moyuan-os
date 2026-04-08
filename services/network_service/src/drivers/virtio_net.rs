// virtio-net网卡驱动

use crate::NetworkError;
use super::NetworkDriver;

// virtio设备ID
const VIRTIO_DEVICE_ID_NET: u16 = 0x1000;

// virtio寄存器地址
const VIRTIO_REG_MAGIC: u32 = 0x0000; // 魔数
const VIRTIO_REG_VERSION: u32 = 0x0004; // 版本
const VIRTIO_REG_DEVICE_ID: u32 = 0x0008; // 设备ID
#[allow(dead_code)]
const VIRTIO_REG_VENDOR_ID: u32 = 0x000C; // 厂商ID
const VIRTIO_REG_DEVICE_FEATURES: u32 = 0x0010; // 设备特性
const VIRTIO_REG_DRIVER_FEATURES: u32 = 0x0020; // 驱动特性
const VIRTIO_REG_QUEUE_SELECT: u32 = 0x0030; // 队列选择
const VIRTIO_REG_QUEUE_SIZE: u32 = 0x0034; // 队列大小
const VIRTIO_REG_QUEUE_ADDRESS: u32 = 0x0038; // 队列地址
const VIRTIO_REG_QUEUE_READY: u32 = 0x0044; // 队列就绪
#[allow(dead_code)]
const VIRTIO_REG_INTERRUPT_STATUS: u32 = 0x0060; // 中断状态
#[allow(dead_code)]
const VIRTIO_REG_INTERRUPT_ACK: u32 = 0x0064; // 中断确认
const VIRTIO_REG_STATUS: u32 = 0x0070; // 设备状态

// virtio设备状态
const VIRTIO_STATUS_RESET: u8 = 0x00;
const VIRTIO_STATUS_ACKNOWLEDGE: u8 = 0x01;
const VIRTIO_STATUS_DRIVER: u8 = 0x02;
const VIRTIO_STATUS_DRIVER_OK: u8 = 0x04;
const VIRTIO_STATUS_FEATURES_OK: u8 = 0x08;

// virtio队列描述符
#[repr(C)]
struct VirtqDesc {
    addr: u64,      // 缓冲区地址
    len: u32,       // 缓冲区长度
    flags: u16,     // 标志
    next: u16,      // 下一个描述符
}

// virtio队列可用环
#[repr(C)]
struct VirtqAvail {
    flags: u16,     // 标志
    idx: u16,       // 索引
    ring: [u16; 32], // 可用描述符环
}

// virtio队列使用环
#[repr(C)]
struct VirtqUsed {
    flags: u16,     // 标志
    idx: u16,       // 索引
    ring: [VirtqUsedElem; 32], // 使用描述符环
}

// virtio队列使用元素
#[repr(C)]
struct VirtqUsedElem {
    id: u32,        // 描述符ID
    len: u32,       // 使用长度
}

// virtio-net配置空间
#[allow(dead_code)]
#[repr(C)]
struct VirtioNetConfig {
    mac: [u8; 6],   // MAC地址
    status: u16,    // 状态
    max_virtqueue_pairs: u16, // 最大队列对
    mtu: u16,       // MTU
}

// virtio-net驱动结构
pub struct VirtioNetDriver {
    base_addr: u64,
    mac_address: [u8; 6],
    tx_queue: VirtqDesc,
    rx_queue: VirtqDesc,
    tx_avail: VirtqAvail,
    rx_avail: VirtqAvail,
    #[allow(dead_code)]
    tx_used: VirtqUsed,
    rx_used: VirtqUsed,
    tx_buffer: [u8; 32 * 1514], // 32个缓冲区，每个最大1514字节
    rx_buffer: [u8; 32 * 1514], // 32个缓冲区，每个最大1514字节
    tx_head: u16,
    rx_head: u16,
    ready: bool,
}

impl VirtioNetDriver {
    // 创建新的virtio-net驱动实例
    pub fn new() -> Result<Self, NetworkError> {
        // 假设virtio-net设备在0x10001000地址
        let base_addr = 0x10001000;
        
        Ok(Self {
            base_addr,
            mac_address: [0; 6],
            tx_queue: VirtqDesc { addr: 0, len: 0, flags: 0, next: 0 },
            rx_queue: VirtqDesc { addr: 0, len: 0, flags: 0, next: 0 },
            tx_avail: VirtqAvail { flags: 0, idx: 0, ring: [0; 32] },
            rx_avail: VirtqAvail { flags: 0, idx: 0, ring: [0; 32] },
            tx_used: VirtqUsed { flags: 0, idx: 0, ring: core::array::from_fn(|_| VirtqUsedElem { id: 0, len: 0 }) },
            rx_used: VirtqUsed { flags: 0, idx: 0, ring: core::array::from_fn(|_| VirtqUsedElem { id: 0, len: 0 }) },
            tx_buffer: [0; 32 * 1514],
            rx_buffer: [0; 32 * 1514],
            tx_head: 0,
            rx_head: 0,
            ready: false,
        })
    }
    
    // 读取32位寄存器
    fn read_reg32(&self, offset: u32) -> u32 {
        unsafe {
            core::ptr::read_volatile((self.base_addr + offset as u64) as *const u32)
        }
    }
    
    // 写入32位寄存器
    fn write_reg32(&self, offset: u32, value: u32) {
        unsafe {
            core::ptr::write_volatile((self.base_addr + offset as u64) as *mut u32, value);
        }
    }
    
    // 读取8位寄存器
    #[allow(dead_code)]
    fn read_reg8(&self, offset: u32) -> u8 {
        unsafe {
            core::ptr::read_volatile((self.base_addr + offset as u64) as *const u8)
        }
    }
    
    // 写入8位寄存器
    fn write_reg8(&self, offset: u32, value: u8) {
        unsafe {
            core::ptr::write_volatile((self.base_addr + offset as u64) as *mut u8, value);
        }
    }
    
    // 读取配置空间
    fn read_config(&self, offset: u32, data: &mut [u8]) {
        unsafe {
            let config_addr = self.base_addr + 0x1000;
            core::ptr::copy((config_addr + offset as u64) as *const u8, data.as_mut_ptr(), data.len());
        }
    }
}

impl NetworkDriver for VirtioNetDriver {
    // 初始化驱动
    fn init(&mut self) -> Result<(), NetworkError> {
        // 检查魔数和版本
        let magic = self.read_reg32(VIRTIO_REG_MAGIC);
        let version = self.read_reg32(VIRTIO_REG_VERSION);
        let device_id = self.read_reg32(VIRTIO_REG_DEVICE_ID);
        
        if magic != 0x74726976 || version != 1 || device_id != VIRTIO_DEVICE_ID_NET as u32 {
            return Err(NetworkError::NotSupported);
        }
        
        // 重置设备
        self.write_reg8(VIRTIO_REG_STATUS, VIRTIO_STATUS_RESET);
        
        // 确认设备
        self.write_reg8(VIRTIO_REG_STATUS, VIRTIO_STATUS_ACKNOWLEDGE);
        self.write_reg8(VIRTIO_REG_STATUS, VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER);
        
        // 协商特性
        let _device_features = self.read_reg32(VIRTIO_REG_DEVICE_FEATURES);
        // 只启用基本特性
        self.write_reg32(VIRTIO_REG_DRIVER_FEATURES, 0);
        
        // 确认特性
        self.write_reg8(VIRTIO_REG_STATUS, VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER | VIRTIO_STATUS_FEATURES_OK);
        
        // 读取MAC地址
        let mut mac_addr = [0; 6];
        self.read_config(0, &mut mac_addr);
        self.mac_address = mac_addr;
        
        // 初始化发送队列
        self.write_reg32(VIRTIO_REG_QUEUE_SELECT, 0); // 选择发送队列
        self.write_reg32(VIRTIO_REG_QUEUE_SIZE, 32); // 设置队列大小
        let tx_queue_addr = &self.tx_queue as *const VirtqDesc as u64;
        self.write_reg32(VIRTIO_REG_QUEUE_ADDRESS, tx_queue_addr as u32);
        self.write_reg32(VIRTIO_REG_QUEUE_READY, 1); // 标记队列为就绪
        
        // 初始化接收队列
        self.write_reg32(VIRTIO_REG_QUEUE_SELECT, 1); // 选择接收队列
        self.write_reg32(VIRTIO_REG_QUEUE_SIZE, 32); // 设置队列大小
        let rx_queue_addr = &self.rx_queue as *const VirtqDesc as u64;
        self.write_reg32(VIRTIO_REG_QUEUE_ADDRESS, rx_queue_addr as u32);
        self.write_reg32(VIRTIO_REG_QUEUE_READY, 1); // 标记队列为就绪
        
        // 初始化接收缓冲区
        for i in 0..32 {
            let buffer_addr = &self.rx_buffer[i * 1514] as *const u8 as u64;
            self.rx_queue.addr = buffer_addr;
            self.rx_queue.len = 1514;
            self.rx_queue.flags = 0;
            self.rx_queue.next = 0;
            
            // 添加到可用环
            self.rx_avail.ring[i] = i as u16;
        }
        self.rx_avail.idx = 32;
        
        // 完成初始化
        self.write_reg8(VIRTIO_REG_STATUS, VIRTIO_STATUS_ACKNOWLEDGE | VIRTIO_STATUS_DRIVER | VIRTIO_STATUS_FEATURES_OK | VIRTIO_STATUS_DRIVER_OK);
        
        self.ready = true;
        Ok(())
    }
    
    // 发送数据包
    fn send_packet(&mut self, data: &[u8]) -> Result<(), NetworkError> {
        if !self.ready {
            return Err(NetworkError::NotSupported);
        }
        
        if data.len() > 1514 {
            return Err(NetworkError::InvalidArgument);
        }
        
        // 复制数据到发送缓冲区
        let buffer_start = self.tx_head as usize * 1514;
        unsafe {
            core::ptr::copy(data.as_ptr(), &mut self.tx_buffer[buffer_start], data.len());
        }
        
        // 设置发送描述符
        let buffer_addr = &self.tx_buffer[buffer_start] as *const u8 as u64;
        self.tx_queue.addr = buffer_addr;
        self.tx_queue.len = data.len() as u32;
        self.tx_queue.flags = 0;
        self.tx_queue.next = 0;
        
        // 添加到可用环
        self.tx_avail.ring[self.tx_head as usize] = self.tx_head;
        self.tx_head = (self.tx_head + 1) % 32;
        self.tx_avail.idx = self.tx_head;
        
        Ok(())
    }
    
    // 接收数据包
    fn receive_packet(&mut self, buffer: &mut [u8]) -> Result<usize, NetworkError> {
        if !self.ready {
            return Err(NetworkError::NotSupported);
        }
        
        // 检查是否有新的数据包
        if self.rx_used.idx == 0 {
            return Err(NetworkError::ResourceBusy);
        }
        
        // 获取使用的描述符
        let used_idx = (self.rx_used.idx - 1) % 32;
        let used_elem = &self.rx_used.ring[used_idx as usize];
        let length = used_elem.len as usize;
        
        if length > buffer.len() {
            return Err(NetworkError::InvalidArgument);
        }
        
        // 复制数据到缓冲区
        let buffer_start = used_elem.id as usize * 1514;
        unsafe {
            core::ptr::copy(&self.rx_buffer[buffer_start], buffer.as_mut_ptr(), length);
        }
        
        // 重置接收描述符
        let buffer_addr = &self.rx_buffer[buffer_start] as *const u8 as u64;
        self.rx_queue.addr = buffer_addr;
        self.rx_queue.len = 1514;
        self.rx_queue.flags = 0;
        self.rx_queue.next = 0;
        
        // 添加回可用环
        self.rx_avail.ring[self.rx_head as usize] = used_elem.id as u16;
        self.rx_head = (self.rx_head + 1) % 32;
        self.rx_avail.idx = self.rx_head;
        
        // 确认使用
        self.rx_used.idx = 0;
        
        Ok(length)
    }
    
    // 获取MAC地址
    fn get_mac_address(&self) -> [u8; 6] {
        self.mac_address
    }
    
    // 检查驱动是否就绪
    fn is_ready(&self) -> bool {
        self.ready
    }
}