// e1000网卡驱动

use crate::NetworkError;
use super::NetworkDriver;

// e1000寄存器地址
const E1000_REG_MAC: u32 = 0x00000; // MAC地址寄存器
const E1000_REG_CTRL: u32 = 0x00008; // 控制寄存器
#[allow(dead_code)]
const E1000_REG_STATUS: u32 = 0x0000C; // 状态寄存器
const E1000_REG_TDBAL: u32 = 0x03800; // 发送描述符基地址低32位
const E1000_REG_TDBAH: u32 = 0x03804; // 发送描述符基地址高32位
const E1000_REG_TDLEN: u32 = 0x03808; // 发送描述符长度
const E1000_REG_TDH: u32 = 0x03810; // 发送描述符头
const E1000_REG_TDT: u32 = 0x03818; // 发送描述符尾
const E1000_REG_RDBAL: u32 = 0x02800; // 接收描述符基地址低32位
const E1000_REG_RDBAH: u32 = 0x02804; // 接收描述符基地址高32位
const E1000_REG_RDLEN: u32 = 0x02808; // 接收描述符长度
const E1000_REG_RDH: u32 = 0x02810; // 接收描述符头
const E1000_REG_RDT: u32 = 0x02818; // 接收描述符尾

// 发送描述符
#[repr(C)]
struct TxDesc {
    buffer_addr: u64,
    length: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

// 接收描述符
#[repr(C)]
struct RxDesc {
    buffer_addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

// e1000驱动结构
pub struct E1000Driver {
    base_addr: u64,
    mac_address: [u8; 6],
    tx_descriptors: [TxDesc; 32],
    rx_descriptors: [RxDesc; 32],
    tx_buffer: [u8; 32 * 1514], // 32个缓冲区，每个最大1514字节
    rx_buffer: [u8; 32 * 1514], // 32个缓冲区，每个最大1514字节
    #[allow(dead_code)]
    tx_head: u8,
    tx_tail: u8,
    rx_head: u8,
    #[allow(dead_code)]
    rx_tail: u8,
    ready: bool,
}

impl E1000Driver {
    // 创建新的e1000驱动实例
    pub fn new() -> Result<Self, NetworkError> {
        // 假设e1000网卡在0x10000000地址
        let base_addr = 0x10000000;
        
        Ok(Self {
            base_addr,
            mac_address: [0; 6],
            tx_descriptors: core::array::from_fn(|_| TxDesc { buffer_addr: 0, length: 0, cso: 0, cmd: 0, status: 0, css: 0, special: 0 }),
            rx_descriptors: core::array::from_fn(|_| RxDesc { buffer_addr: 0, length: 0, checksum: 0, status: 0, errors: 0, special: 0 }),
            tx_buffer: [0; 32 * 1514],
            rx_buffer: [0; 32 * 1514],
            tx_head: 0,
            tx_tail: 0,
            rx_head: 0,
            rx_tail: 0,
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
    
    // 读取64位寄存器
    #[allow(dead_code)]
    fn read_reg64(&self, offset: u32) -> u64 {
        unsafe {
            core::ptr::read_volatile((self.base_addr + offset as u64) as *const u64)
        }
    }
    
    // 写入64位寄存器
    #[allow(dead_code)]
    fn write_reg64(&self, offset: u32, value: u64) {
        unsafe {
            core::ptr::write_volatile((self.base_addr + offset as u64) as *mut u64, value);
        }
    }
}

impl NetworkDriver for E1000Driver {
    // 初始化驱动
    fn init(&mut self) -> Result<(), NetworkError> {
        // 读取MAC地址
        for i in 0..6 {
            self.mac_address[i] = self.read_reg32(E1000_REG_MAC + i as u32 * 4) as u8;
        }
        
        // 初始化发送描述符
        let tx_desc_addr = self.tx_descriptors.as_ptr() as u64;
        self.write_reg32(E1000_REG_TDBAL, tx_desc_addr as u32);
        self.write_reg32(E1000_REG_TDBAH, (tx_desc_addr >> 32) as u32);
        self.write_reg32(E1000_REG_TDLEN, (self.tx_descriptors.len() * core::mem::size_of::<TxDesc>()) as u32);
        self.write_reg32(E1000_REG_TDH, 0);
        self.write_reg32(E1000_REG_TDT, 0);
        
        // 初始化发送缓冲区
        for i in 0..32 {
            let buffer_addr = &self.tx_buffer[i * 1514] as *const u8 as u64;
            self.tx_descriptors[i].buffer_addr = buffer_addr;
            self.tx_descriptors[i].status = 1; // 标记为就绪
        }
        
        // 初始化接收描述符
        let rx_desc_addr = self.rx_descriptors.as_ptr() as u64;
        self.write_reg32(E1000_REG_RDBAL, rx_desc_addr as u32);
        self.write_reg32(E1000_REG_RDBAH, (rx_desc_addr >> 32) as u32);
        self.write_reg32(E1000_REG_RDLEN, (self.rx_descriptors.len() * core::mem::size_of::<RxDesc>()) as u32);
        self.write_reg32(E1000_REG_RDH, 0);
        self.write_reg32(E1000_REG_RDT, 31); // 32个描述符，所以是31
        
        // 初始化接收缓冲区
        for i in 0..32 {
            let buffer_addr = &self.rx_buffer[i * 1514] as *const u8 as u64;
            self.rx_descriptors[i].buffer_addr = buffer_addr;
        }
        
        // 启用网卡
        let ctrl = self.read_reg32(E1000_REG_CTRL);
        self.write_reg32(E1000_REG_CTRL, ctrl | 1); // 启用网卡
        
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
        
        // 检查发送队列是否已满
        if self.tx_descriptors[self.tx_tail as usize].status == 0 {
            return Err(NetworkError::ResourceBusy);
        }
        
        // 复制数据到发送缓冲区
        let buffer_start = self.tx_tail as usize * 1514;
        unsafe {
            core::ptr::copy(data.as_ptr(), &mut self.tx_buffer[buffer_start], data.len());
        }
        
        // 更新发送描述符
        self.tx_descriptors[self.tx_tail as usize].length = data.len() as u16;
        self.tx_descriptors[self.tx_tail as usize].cmd = 0x04; // 发送命令
        self.tx_descriptors[self.tx_tail as usize].status = 0; // 标记为未就绪
        
        // 更新发送指针
        self.tx_tail = (self.tx_tail + 1) % 32;
        self.write_reg32(E1000_REG_TDT, self.tx_tail as u32);
        
        Ok(())
    }
    
    // 接收数据包
    fn receive_packet(&mut self, buffer: &mut [u8]) -> Result<usize, NetworkError> {
        if !self.ready {
            return Err(NetworkError::NotSupported);
        }
        
        // 检查是否有新的数据包
        if self.rx_descriptors[self.rx_head as usize].status & 1 == 0 {
            return Err(NetworkError::ResourceBusy);
        }
        
        // 获取数据包长度
        let length = self.rx_descriptors[self.rx_head as usize].length as usize;
        if length > buffer.len() {
            return Err(NetworkError::InvalidArgument);
        }
        
        // 复制数据到缓冲区
        let buffer_start = self.rx_head as usize * 1514;
        unsafe {
            core::ptr::copy(&self.rx_buffer[buffer_start], buffer.as_mut_ptr(), length);
        }
        
        // 重置接收描述符
        self.rx_descriptors[self.rx_head as usize].status = 0;
        
        // 更新接收指针
        self.rx_head = (self.rx_head + 1) % 32;
        self.write_reg32(E1000_REG_RDH, self.rx_head as u32);
        
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