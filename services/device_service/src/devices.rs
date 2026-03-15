use super::{Device, DeviceError, DeviceNumber, CharacterDevice, DisplayDevice};
use core::ffi::c_void;

// 显示模式枚举
#[derive(Clone, Copy)]
pub enum DisplayMode {
    Text,
    Graphics,
}

// 键盘操作接口
pub mod keyboard {
    pub fn init() {
        // 调用内核的键盘初始化函数
        unsafe {
            extern "C" { fn keyboard_init(); }
            keyboard_init();
        }
    }
    
    pub fn read_key() -> Option<char> {
        // 调用内核的键盘读取函数，使用 u8 作为 FFI 安全类型
        unsafe {
            extern "C" { fn keyboard_read_key() -> u8; }
            let key = keyboard_read_key();
            if key != 0 {
                Some(key as char)
            } else {
                None
            }
        }
    }
    
    pub fn has_key() -> bool {
        // 调用内核的键盘检查函数
        unsafe {
            extern "C" { fn keyboard_has_key() -> bool; }
            keyboard_has_key()
        }
    }
}

// VGA操作接口
pub mod vga {
    use super::DisplayMode;
    
    pub fn init() {
        // 调用内核的VGA初始化函数
        unsafe {
            extern "C" { fn vga_init(); }
            vga_init();
        }
    }
    
    pub fn set_mode(mode: DisplayMode) {
        // 调用内核的VGA设置模式函数
        unsafe {
            extern "C" { fn vga_set_mode(mode: u8); }
            let mode_code = match mode {
                DisplayMode::Text => 0,
                DisplayMode::Graphics => 1,
            };
            vga_set_mode(mode_code);
        }
    }
    
    pub fn print_char(c: char, x: usize, y: usize, fg_color: u8, bg_color: u8) {
        // 调用内核的VGA打印字符函数
        unsafe {
            extern "C" { fn vga_print_char(c: u8, x: usize, y: usize, fg_color: u8, bg_color: u8); }
            vga_print_char(c as u8, x, y, fg_color, bg_color);
        }
    }
    
    pub fn clear_screen() {
        // 调用内核的VGA清屏函数
        unsafe {
            extern "C" { fn vga_clear_screen(); }
            vga_clear_screen();
        }
    }
    
    pub fn put_char(c: char) {
        // 调用内核的VGA显示字符函数
        unsafe {
            extern "C" { fn vga_put_char(c: u8); }
            vga_put_char(c as u8);
        }
    }
    
    pub fn set_cursor_pos(x: usize, y: usize) {
        // 调用内核的VGA设置光标位置函数
        unsafe {
            extern "C" { fn vga_set_cursor_pos(x: usize, y: usize); }
            vga_set_cursor_pos(x, y);
        }
    }
    
    pub fn set_framebuffer(addr: u64, width: usize, height: usize, pitch: usize, bpp: usize) {
        // 调用内核的VGA设置帧缓冲区函数
        unsafe {
            extern "C" { fn vga_set_framebuffer(addr: u64, width: usize, height: usize, pitch: usize, bpp: usize); }
            vga_set_framebuffer(addr, width, height, pitch, bpp);
        }
    }
}

// 键盘设备实现
pub struct KeyboardDevice {
    device_number: DeviceNumber,
}

impl Default for KeyboardDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardDevice {
    pub const fn new() -> Self {
        Self {
            device_number: DeviceNumber { major: 1, minor: 0 },
        }
    }
}

impl Device for KeyboardDevice {
    fn init(&self) -> Result<(), DeviceError> {
        keyboard::init();
        Ok(())
    }
    
    fn read(&self, buffer: &mut [u8]) -> Result<usize, DeviceError> {
        let mut count = 0;
        for byte in buffer.iter_mut() {
            if let Some(c) = keyboard::read_key() {
                *byte = c as u8;
                count += 1;
            } else {
                break;
            }
        }
        Ok(count)
    }
    
    fn write(&self, _buffer: &[u8]) -> Result<usize, DeviceError> {
        Err(DeviceError::NotSupported)
    }
    
    fn ioctl(&self, cmd: u32, _args: *mut c_void) -> Result<i32, DeviceError> {
        match cmd {
            0 => Ok(keyboard::has_key() as i32),
            _ => Err(DeviceError::NotSupported),
        }
    }
    
    fn name(&self) -> &'static str {
        "keyboard"
    }
    
    fn device_number(&self) -> DeviceNumber {
        self.device_number
    }
}

impl CharacterDevice for KeyboardDevice {
    fn read_char(&self) -> Option<char> {
        keyboard::read_key()
    }
    
    fn has_input(&self) -> bool {
        keyboard::has_key()
    }
}

// VGA设备实现
pub struct VGADevice {
    device_number: DeviceNumber,
}

impl Default for VGADevice {
    fn default() -> Self {
        Self::new()
    }
}

impl VGADevice {
    pub const fn new() -> Self {
        Self {
            device_number: DeviceNumber { major: 2, minor: 0 },
        }
    }
}

impl Device for VGADevice {
    fn init(&self) -> Result<(), DeviceError> {
        vga::init();
        Ok(())
    }
    
    fn read(&self, _buffer: &mut [u8]) -> Result<usize, DeviceError> {
        Err(DeviceError::NotSupported)
    }
    
    fn write(&self, buffer: &[u8]) -> Result<usize, DeviceError> {
        for &byte in buffer {
            if byte == b'\n' {
                vga::put_char('\n');
            } else if byte.is_ascii_graphic() || byte == b' ' {
                vga::put_char(byte as char);
            }
        }
        Ok(buffer.len())
    }
    
    fn ioctl(&self, cmd: u32, args: *mut c_void) -> Result<i32, DeviceError> {
        match cmd {
            0 => {
                // 清屏
                vga::clear_screen();
                Ok(0)
            }
            1 => {
                // 设置光标位置
                let pos = unsafe { &*(args as *const (usize, usize)) };
                vga::set_cursor_pos(pos.0, pos.1);
                Ok(0)
            }
            2 => {
                // 设置显示模式
                let mode = unsafe { &*(args as *const DisplayMode) };
                vga::set_mode(*mode);
                Ok(0)
            }
            3 => {
                // 设置帧缓冲区
                let fb_info = unsafe { &*(args as *const (u64, usize, usize, usize, usize)) };
                vga::set_framebuffer(fb_info.0, fb_info.1, fb_info.2, fb_info.3, fb_info.4);
                Ok(0)
            }
            _ => Err(DeviceError::NotSupported),
        }
    }
    
    fn name(&self) -> &'static str {
        "vga"
    }
    
    fn device_number(&self) -> DeviceNumber {
        self.device_number
    }
}

impl DisplayDevice for VGADevice {
    type DisplayMode = DisplayMode;
    
    fn set_mode(&self, mode: Self::DisplayMode) -> Result<(), DeviceError> {
        vga::set_mode(mode);
        Ok(())
    }
    
    fn print_char(&self, c: char, x: usize, y: usize, fg_color: u8, bg_color: u8) -> Result<(), DeviceError> {
        vga::print_char(c, x, y, fg_color, bg_color);
        Ok(())
    }
    
    fn clear_screen(&self) -> Result<(), DeviceError> {
        vga::clear_screen();
        Ok(())
    }
}

// 全局设备实例
pub static KEYBOARD_DEVICE: KeyboardDevice = KeyboardDevice::new();
pub static VGA_DEVICE: VGADevice = VGADevice::new();
