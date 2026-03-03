#![no_std]

use core::ffi::c_void;

// 设备错误类型
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DeviceError {
    NotReady,      // 设备未就绪
    ReadFailed,    // 读取失败
    WriteFailed,   // 写入失败
    NotSupported,  // 不支持的操作
    InvalidParam,  // 无效参数
}

// 设备号结构
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceNumber {
    pub major: u8,    // 主设备号（8位）
    pub minor: u16,   // 次设备号（16位）
}

// 设备类型枚举
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DeviceType {
    CharacterDevice,  // 字符设备
    DisplayDevice,    // 显示设备
    BlockDevice,      // 块设备
}

// 设备抽象接口
pub trait Device {
    // 设备初始化
    fn init(&self) -> Result<(), DeviceError>;
    
    // 数据读取
    fn read(&self, buffer: &mut [u8]) -> Result<usize, DeviceError>;
    
    // 数据写入
    fn write(&self, buffer: &[u8]) -> Result<usize, DeviceError>;
    
    // 设备控制
    fn ioctl(&self, cmd: u32, args: *mut c_void) -> Result<i32, DeviceError>;
    
    // 获取设备名称
    fn name(&self) -> &'static str;
    
    // 获取设备号
    fn device_number(&self) -> DeviceNumber;
}

// 字符设备 trait
pub trait CharacterDevice: Device {
    // 读取一个字符
    fn read_char(&self) -> Option<char>;
    
    // 检查是否有可用输入
    fn has_input(&self) -> bool;
}

// 显示设备 trait
pub trait DisplayDevice: Device {
    // 显示模式枚举
    type DisplayMode;
    
    // 设置显示模式
    fn set_mode(&self, mode: Self::DisplayMode) -> Result<(), DeviceError>;
    
    // 在指定位置显示字符
    fn print_char(&self, c: char, x: usize, y: usize, fg_color: u8, bg_color: u8) -> Result<(), DeviceError>;
    
    // 清屏
    fn clear_screen(&self) -> Result<(), DeviceError>;
}

// 块设备 trait
pub trait BlockDevice: Device {
    // 读取块
    fn read_block(&self, block_number: u64, buffer: &mut [u8]) -> Result<usize, DeviceError>;
    
    // 写入块
    fn write_block(&self, block_number: u64, buffer: &[u8]) -> Result<usize, DeviceError>;
    
    // 获取块大小
    fn block_size(&self) -> usize;
}

// 设备注册表
pub struct DeviceRegistry {
    devices: [Option<&'static dyn Device>; 256], // 最多支持256个设备
    next_minor: [u16; 256], // 每个主设备号对应的下一个次设备号
}

impl DeviceRegistry {
    // 创建新的设备注册表
    pub const fn new() -> Self {
        Self {
            devices: [None; 256],
            next_minor: [0; 256],
        }
    }
    
    // 注册设备
    pub fn register_device(&mut self, device: &'static dyn Device) -> Result<DeviceNumber, DeviceError> {
        let device_num = device.device_number();
        let index = (device_num.major as usize) * 256 + (device_num.minor as usize);
        
        if index >= self.devices.len() {
            return Err(DeviceError::InvalidParam);
        }
        
        if self.devices[index].is_some() {
            return Err(DeviceError::InvalidParam);
        }
        
        self.devices[index] = Some(device);
        Ok(device_num)
    }
    
    // 注销设备
    pub fn unregister_device(&mut self, device_num: DeviceNumber) -> Result<(), DeviceError> {
        let index = (device_num.major as usize) * 256 + (device_num.minor as usize);
        
        if index >= self.devices.len() {
            return Err(DeviceError::InvalidParam);
        }
        
        if self.devices[index].is_none() {
            return Err(DeviceError::InvalidParam);
        }
        
        self.devices[index] = None;
        Ok(())
    }
    
    // 根据设备号查找设备
    pub fn find_device(&self, device_num: DeviceNumber) -> Option<&'static dyn Device> {
        let index = (device_num.major as usize) * 256 + (device_num.minor as usize);
        
        if index < self.devices.len() {
            self.devices[index]
        } else {
            None
        }
    }
    
    // 遍历所有设备
    pub fn for_each_device<F>(&self, mut f: F) where F: FnMut(&dyn Device) {
        for device_option in self.devices.iter() {
            if let Some(device) = device_option {
                f(*device);
            }
        }
    }
}

// 全局设备注册表
static mut DEVICE_REGISTRY: DeviceRegistry = DeviceRegistry::new();

// 导出设备实现
pub mod devices;

// 初始化设备服务
pub fn init() {
    // 初始化设备注册表
    unsafe {
        // 注册键盘设备
        if let Ok(_device_num) = DEVICE_REGISTRY.register_device(&devices::KEYBOARD_DEVICE) {
            // 实际实现中将使用适当的日志或打印机制
            devices::KEYBOARD_DEVICE.init().unwrap();
        }
        
        // 注册VGA设备
        if let Ok(_device_num) = DEVICE_REGISTRY.register_device(&devices::VGA_DEVICE) {
            // 实际实现中将使用适当的日志或打印机制
            devices::VGA_DEVICE.init().unwrap();
        }
    }
}

// 注册设备
pub fn register_device(device: &'static dyn Device) -> Result<DeviceNumber, DeviceError> {
    unsafe {
        DEVICE_REGISTRY.register_device(device)
    }
}

// 注销设备
pub fn unregister_device(device_num: DeviceNumber) -> Result<(), DeviceError> {
    unsafe {
        DEVICE_REGISTRY.unregister_device(device_num)
    }
}

// 查找设备
pub fn find_device(device_num: DeviceNumber) -> Option<&'static dyn Device> {
    unsafe {
        DEVICE_REGISTRY.find_device(device_num)
    }
}

// 遍历所有设备
pub fn for_each_device<F>(f: F) where F: FnMut(&dyn Device) {
    unsafe {
        DEVICE_REGISTRY.for_each_device(f)
    }
}
