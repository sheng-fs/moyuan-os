// 控制台模块

pub mod serial;
pub mod vga;
pub mod keyboard;

/// 初始化控制台
#[allow(dead_code)]
pub fn init() {
    serial::init();
    vga::init();
    keyboard::init();
}

/// 打印字符串
pub fn print(args: fmt::Arguments) {
    serial::print(args);
    vga::print(args);
}

/// 打印宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::print(format_args!($($arg)*));
    };
}

/// 打印换行宏
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}

// 导入格式化模块
use core::fmt;